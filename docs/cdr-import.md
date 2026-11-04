# Independent CDR import

Reviewed: 2026-11-04. Experimental fork feature; upstream parity scores are unchanged.

The `vectorcraft-cdr` crate reads `.cdr` through the independent Rust reader in
[TraceDraw](https://github.com/tracedraw/tracedraw), pinned to
`b98a041a6e26ed925991bcab7a788a34812339a7`. Its SVG writer adapts each visible page into
VectorCraft's existing editable SVG importer. CorelDRAW, COM, an external converter,
network access and the earlier PowerShell/PDF bridge are not required at runtime.

## Entry points

CDR is a read-only entry in `document.formats`, the shared Open/Place filters and
`document.open`. Desktop, browser, CLI, JSON control and MCP use that same engine path.
For example, with a build of this fork:

```sh
vectorcraft-cli info drawing.cdr
vectorcraft-cli convert drawing.cdr drawing.vectorcraft
vectorcraft-cli convert drawing.cdr drawing.pdf
```

The installed upstream 0.21.0 executable remains separate and does not acquire this
feature merely because this source checkout exists.

## Fidelity and budgets

- This is an import, not a lossless CDR editor. Saving writes a VectorCraft copy or
  another supported export format; it never writes CDR over the original.
- Each page becomes a separate artboard at its physical size. Pages are arranged in a
  horizontal spread with a 10 mm gap. SVG resource ids are namespaced by page.
- The upstream reader understands multiple CDR generations; compatibility is limited
  to what that reader and the SVG adapter can represent. Unknown objects and warnings
  are surfaced. Every import includes a limitation note.
- The SVG step may omit hidden art and layer flags, convert spot inks, substitute fonts,
  flatten or approximate effects and lose live editing relationships. Check artwork and
  colours before production. Embedded CDR previews are not used as a substitute for decoding.
- Input is capped at 64 MiB, ZIP members at 16,384 / 256 MiB declared and actual uncompressed total,
  pages at 256 and generated SVG at 128 MiB. The ZIP data-file index is limited to
  1 MiB / 16,384 lines before the reader allocates its strings. Reader and SVG limits
  apply as well. Browser builds select getrandom's `wasm_js` backend explicitly.
- Synthetic tests cover RIFF and ZIP generations, page size, multiple pages, malformed
  containers and declared ZIP expansion budgets. The engine has a CDR random-input
  property test. Customer drawings are local-only and must never be committed.

The dependency's public API and source have been inspected for this integration. No
proprietary executable was disassembled or copied. Dependency provenance and the narrow
Brazilian software-law review are in [legal-review-cdr.md](legal-review-cdr.md).

## Validation record

Verified locally on Windows x64, 2026-11-04:

- `cargo test -p vectorcraft-cdr --lib`: 5 passed (four CDR versions, multiple pages,
  truncated data, ZIP expansion and index-allocation limits).
- `cargo clippy -p vectorcraft-cdr --all-targets -- -D warnings`: passed.
- `cargo build -p vectorcraft-cli`: passed. The resulting executable read a user-authorized
  local CDR 23 sample and wrote native VectorCraft artwork and a rendered PNG.
- That local sample kept one 148 x 210 mm page and its full 6144 x 8192 pixel image,
  not the 180-pixel embedded page preview. Its source SHA-256 was unchanged. The sample,
  its contents, previews, paths and hashes are not published in this repository.
- A fresh headless MCP server advertised CDR and opened that file through `command_run`
  / `document.open`, returning `format: cdr` and the explicit limitation note.
- Asset attribution, user-visible brand scan and dependency layering checks passed.
- The full `cargo xtask ci` was attempted: formatting passed, then the broad workspace
  Clippy build was stopped to preserve local disk space. This is not a full CI pass;
  the complete shared GitHub gate remains required before a release.

The engine regression and browser-target checks are recorded separately when complete.
Passing an import of a bitmap-only drawing does not establish fidelity for editable text,
gradients, clip containers, spot inks or complex effects. The installed official desktop
app is separate from this checkout; an experimental CLI/MCP build is a different program.
