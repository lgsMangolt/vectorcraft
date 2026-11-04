//! CDR is reachable through the same commands used by desktop, CLI and MCP.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
use vectorcraft_engine::Session;

#[test]
fn cdr_is_advertised_read_only_and_bad_import_is_transactional() {
    let mut session = Session::new();
    let formats = session.execute("document.formats", &json!({})).unwrap();
    assert!(formats.to_string().contains("cdr"));
    let format = vectorcraft_engine::cmd::fileio::format("cdr").unwrap();
    assert!(format.read);
    assert!(!format.write);
    session.execute("file.new", &json!({"width": 120, "height": 80})).unwrap();
    let before = session.doc().unwrap().doc.clone();
    let data = vectorcraft_format::base64_encode(b"not a drawing");
    assert!(session.execute("document.open", &json!({"name": "broken.cdr", "dataBase64": data})).is_err());
    assert_eq!(session.doc().unwrap().doc, before);
}
