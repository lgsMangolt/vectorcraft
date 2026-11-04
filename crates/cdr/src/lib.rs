//! Read CDR independently, then adapt the visible drawing through SVG into the native document.
//! No CorelDRAW installation, COM, network requests or external conversion process is used.
#![forbid(unsafe_code)]

use std::fmt::Write;
use std::io::{Cursor, Read};

use vectorcraft_doc::Document;

/// Compressed input budget, checked before the independent reader runs.
pub const MAX_INPUT_BYTES: usize = 64 << 20;
const MAX_UNPACKED_BYTES: u64 = 256 << 20;
const MAX_ZIP_MEMBERS: usize = 16_384;
const MAX_DATA_FILE_LIST: u64 = 1 << 20;
const MAX_PAGES: usize = 256;
const MAX_SVG_BYTES: usize = 128 << 20;
const MAX_SPREAD_MM: f64 = 1_000_000.0;

/// This first adapter is an import, not a lossless CDR editing round trip.
pub const IMPORT_NOTE: &str = "CDR: imported independently through an SVG adapter. Hidden artwork, layer properties, spot inks, embedded fonts and advanced/live effects may be omitted, flattened or approximated. Check the drawing before production and save a new VectorCraft copy; CDR export is not supported.";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("CDR: {0}")]
    Invalid(String),
    #[error("CDR exceeds the import budget: {0}")]
    Limit(&'static str),
}

/// A decoded document and every import limitation reported by either reader.
pub struct Imported {
    pub document: Document,
    pub warnings: Vec<String>,
}

/// Cheap recognition of old RIFF/WL files. ZIP CDR files also use their `.cdr` extension.
pub fn sniff(bytes: &[u8]) -> bool {
    (bytes.starts_with(b"RIFF") && bytes.get(8..11).is_some_and(|tag| tag.eq_ignore_ascii_case(b"cdr"))) || tracedraw_cdr::wl::is_wl(bytes)
}

/// Read a CDR without modifying its bytes or invoking another application.
pub fn import(bytes: &[u8], title: &str) -> Result<Imported, Error> {
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(Error::Limit("input larger than 64 MiB"));
    }
    if bytes.starts_with(b"RIFF") {
        let declared = bytes.get(4..8).and_then(|b| <[u8; 4]>::try_from(b).ok()).map(u32::from_le_bytes);
        if declared.is_none_or(|size| size < 4 || u64::from(size) + 8 > bytes.len() as u64) {
            return Err(Error::Invalid("truncated RIFF container".into()));
        }
    }
    check_archive(bytes)?;
    let (source, report) = tracedraw_cdr::open_bytes(bytes, title).map_err(|e| Error::Invalid(e.to_string()))?;
    if report.pages == 0 {
        return Err(Error::Invalid("no readable drawing pages".into()));
    }
    if source.pages.is_empty() || source.pages.len() > MAX_PAGES {
        return Err(Error::Limit("expected 1 to 256 pages"));
    }
    let mut width = 0.0_f64;
    let mut height = 0.0_f64;
    for page in &source.pages {
        let (w, h) = (page.size.width, page.size.height);
        if !(w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0) {
            return Err(Error::Invalid("invalid page dimensions".into()));
        }
        width += w + 10.0;
        height = height.max(h);
        if width > MAX_SPREAD_MM || height > MAX_SPREAD_MM {
            return Err(Error::Limit("page spread is too large"));
        }
    }
    width -= 10.0;
    // One shared SVG preserves every page's physical size. Namespaced resource ids prevent a
    // gradient/image/clip from a later page resolving against a previous page's definition.
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:inkscape=\"http://www.inkscape.org/namespaces/inkscape\" xmlns:sodipodi=\"http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd\" width=\"{width}mm\" height=\"{height}mm\" viewBox=\"0 0 {width} {height}\"><sodipodi:namedview>"
    );
    let mut x = 0.0;
    for page in &source.pages {
        write!(
            svg,
            "<inkscape:page x=\"{x}\" y=\"0\" width=\"{}\" height=\"{}\" inkscape:label=\"{}\"/>",
            page.size.width,
            page.size.height,
            xml(&page.name)
        )
        .map_err(|e| Error::Invalid(e.to_string()))?;
        x += page.size.width + 10.0;
    }
    svg.push_str("</sodipodi:namedview>");
    x = 0.0;
    for (index, page) in source.pages.iter().enumerate() {
        let exported = tracedraw_io::svg::page_to_svg(&source, index);
        if exported.len() > MAX_SVG_BYTES {
            return Err(Error::Limit("expanded SVG page is too large"));
        }
        let (_, body) = exported.split_once('>').ok_or_else(|| Error::Invalid("missing SVG page header".into()))?;
        let (body, _) = body.rsplit_once("</svg>").ok_or_else(|| Error::Invalid("missing SVG page footer".into()))?;
        let prefix = format!("cdr{index}-");
        let body = body
            .replace("id=\"", &format!("id=\"{prefix}"))
            .replace("url(#", &format!("url(#{prefix}"))
            .replace("href=\"#", &format!("href=\"#{prefix}"));
        if svg.len().saturating_add(body.len()).saturating_add(1024) > MAX_SVG_BYTES {
            return Err(Error::Limit("expanded SVG document is too large"));
        }
        write!(svg, "<g transform=\"translate({x},0)\">{body}</g>").map_err(|e| Error::Invalid(e.to_string()))?;
        x += page.size.width + 10.0;
    }
    svg.push_str("</svg>");
    let (mut document, notes) =
        vectorcraft_svg::import_with(&svg, &vectorcraft_svg::ImportOptions::default()).map_err(|e| Error::Invalid(e.to_string()))?;
    document.title = title.to_string();
    let mut warnings = vec![IMPORT_NOTE.to_string()];
    warnings.extend(report.warnings.into_iter().map(|w| format!("CDR reader: {w}")));
    if report.skipped_objects > 0 {
        warnings.push(format!("CDR reader skipped {} object(s) it could not interpret.", report.skipped_objects));
    }
    warnings.extend(notes);
    Ok(Imported { document, warnings })
}

fn xml(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;")
}

/// The upstream reader's container detection reads its data-file list; bound all declared ZIP
/// members first so even that metadata cannot expand without a budget. Nothing is extracted.
fn check_archive(bytes: &[u8]) -> Result<(), Error> {
    if !bytes.starts_with(b"PK\x03\x04") {
        return Ok(());
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| Error::Invalid(e.to_string()))?;
    if archive.len() > MAX_ZIP_MEMBERS {
        return Err(Error::Limit("too many ZIP members"));
    }
    let mut total = 0_u64;
    let mut actual = 0_u64;
    for index in 0..archive.len() {
        let mut member = archive.by_index(index).map_err(|e| Error::Invalid(e.to_string()))?;
        total = total.checked_add(member.size()).ok_or(Error::Limit("ZIP size overflow"))?;
        if total > MAX_UNPACKED_BYTES {
            return Err(Error::Limit("ZIP members exceed 256 MiB"));
        }
        let remaining = MAX_UNPACKED_BYTES.saturating_sub(actual);
        let read = if member.name_raw() == b"content/dataFileList.dat" {
            let mut list = Vec::new();
            member.by_ref().take(MAX_DATA_FILE_LIST + 1).read_to_end(&mut list).map_err(|e| Error::Invalid(e.to_string()))?;
            if list.len() as u64 > MAX_DATA_FILE_LIST || list.split(|b| *b == b'\n').count() > MAX_ZIP_MEMBERS {
                return Err(Error::Limit("data-file list is too large"));
            }
            list.len() as u64
        } else {
            std::io::copy(&mut member.by_ref().take(remaining + 1), &mut std::io::sink()).map_err(|e| Error::Invalid(e.to_string()))?
        };
        actual = actual.checked_add(read).ok_or(Error::Limit("ZIP size overflow"))?;
        if actual > MAX_UNPACKED_BYTES {
            return Err(Error::Limit("ZIP expansion exceeds 256 MiB"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
