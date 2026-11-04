use super::*;
use tracedraw_core::document::{Layer, Page, Shape, ShapeKind};
use tracedraw_core::geometry::{Rect, Size};
use tracedraw_core::{Color, Document as Source, Fill};

fn sample(pages: usize) -> Source {
    let mut doc = Source::new("Synthetic CDR", Size::new(148.0, 210.0));
    for index in 1..pages {
        let id = doc.ids_mut().page();
        let layer = Layer::new(doc.ids_mut().layer(), format!("Layer {}", index + 1));
        doc.pages.push(Page {
            id,
            name: format!("Page {}", index + 1),
            size: Size::new(100.0, 80.0),
            layers: vec![layer],
            guides: Vec::new(),
            background: None,
        });
    }
    for index in 0..pages {
        let mut shape = Shape::new(doc.ids_mut().shape(), ShapeKind::Rect { rect: Rect::new(5.0, 5.0, 30.0, 20.0), radius: 0.0, corners: None });
        shape.fill = Fill::Solid(Color::rgb8(200, 30, 60));
        shape.stroke = None;
        doc.pages[index].layers[0].shapes.push(shape);
    }
    doc
}

#[test]
fn imports_riff_and_both_zip_generations_as_editable_paths() {
    for version in [12, 15, 23, 27] {
        let bytes = tracedraw_cdr::write::document_to_cdr_version(&sample(1), version);
        let imported = import(&bytes, "Synthetic.cdr").unwrap();
        assert_eq!(imported.document.artboards.len(), 1, "version {version}");
        let board = &imported.document.artboards[0];
        assert!((board.rect.width() * 25.4 / 72.0 - 148.0).abs() < 0.01);
        assert!((board.rect.height() * 25.4 / 72.0 - 210.0).abs() < 0.01);
        let mut paths = 0;
        imported.document.walk(|n| paths += usize::from(matches!(n.kind, vectorcraft_doc::NodeKind::Path { .. })));
        assert!(paths > 0, "version {version}: a vector must not become a preview image");
        assert!(imported.warnings.iter().any(|w| w == IMPORT_NOTE));
    }
}

#[test]
fn every_page_keeps_its_size_and_separate_artboard() {
    let bytes = tracedraw_cdr::write::document_to_cdr_version(&sample(2), 23);
    let imported = import(&bytes, "Two pages.cdr").unwrap();
    let boards = &imported.document.artboards;
    assert_eq!(boards.len(), 2);
    assert!((boards[1].rect.width() * 25.4 / 72.0 - 100.0).abs() < 0.01);
    assert!((boards[1].rect.height() * 25.4 / 72.0 - 80.0).abs() < 0.01);
    assert!(boards[1].rect.x0 > boards[0].rect.x1);
}

#[test]
fn malformed_and_truncated_files_return_errors_without_panicking() {
    let bytes = tracedraw_cdr::write::document_to_cdr_version(&sample(1), 23);
    for input in [&b""[..], &b"not a drawing"[..], &b"RIFF\0\0\0\0CDRO"[..], &bytes[..20], &bytes[..bytes.len() / 2]] {
        let result = std::panic::catch_unwind(|| import(input, "Damaged.cdr"));
        assert!(result.is_ok(), "untrusted input must not panic");
        assert!(result.unwrap().is_err(), "an incomplete container cannot be a valid drawing ({} bytes)", input.len());
    }
}

#[test]
fn archive_budget_is_checked_before_decoding_members() {
    use std::io::Write;
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buffer);
        zip.start_file("content/root.dat", zip::write::SimpleFileOptions::default()).unwrap();
        zip.write_all(b"RIFF\0\0\0\0CDRO").unwrap();
        zip.finish().unwrap();
    }
    let mut bytes = buffer.into_inner();
    let central = bytes.windows(4).position(|w| w == b"PK\x01\x02").unwrap();
    bytes[central + 24..central + 28].copy_from_slice(&((MAX_UNPACKED_BYTES + 1) as u32).to_le_bytes());
    assert!(matches!(import(&bytes, "Oversized.cdr"), Err(Error::Limit(_))));
}

#[test]
fn a_compressed_data_file_list_cannot_allocate_unbounded_strings() {
    use std::io::Write;
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buffer);
        zip.start_file("content/dataFileList.dat", zip::write::SimpleFileOptions::default()).unwrap();
        zip.write_all(&b"x\n".repeat(MAX_ZIP_MEMBERS + 1)).unwrap();
        zip.finish().unwrap();
    }
    assert!(matches!(import(&buffer.into_inner(), "Hostile.cdr"), Err(Error::Limit("data-file list is too large"))));
}
