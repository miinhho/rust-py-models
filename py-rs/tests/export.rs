#![allow(dead_code)]

use py_rs::{ExportError, PY};
use std::collections::HashMap;

#[derive(PY)]
#[py(export_to = "models/Address.py")]
struct Address {
    city: String,
    zip: Option<String>,
}

#[derive(PY)]
enum Status {
    Active,
    #[py(rename = "on-hold")]
    OnHold,
    #[py(rename = "quoted\"\\\u{0001}")]
    Quoted,
}

#[derive(PY)]
enum Event {
    Created { user_id: i64 },
    Moved(Address),
    Deleted,
}

#[derive(PY)]
#[py(export)]
struct User {
    id: u64,
    #[py(rename = "display_name")]
    name: String,
    address: Option<Box<Address>>,
    status: Status,
    events: Vec<Event>,
    labels: HashMap<String, String>,
    #[py(skip)]
    secret: String,
}

#[derive(PY)]
struct Left {
    right: Option<Box<Right>>,
}

#[derive(PY)]
struct Right {
    left: Option<Box<Left>>,
}

#[derive(PY)]
#[py(export)]
struct ZeroArray {
    empty: [u8; 0],
}

#[derive(PY)]
#[allow(non_camel_case_types)]
enum RawVariant {
    r#type,
}

#[derive(PY)]
#[py(export_to = "class/Bad.py")]
struct BadPath;

mod first {
    use py_rs::PY;
    #[derive(PY)]
    #[py(export_to = "first/Thing.py")]
    pub struct Thing;
}

mod second {
    use py_rs::PY;
    #[derive(PY)]
    #[py(export_to = "second/Thing.py")]
    pub struct Thing;
}

#[derive(PY)]
struct Ambiguous {
    first: first::Thing,
    second: second::Thing,
}

#[derive(PY)]
struct VariantName;

#[derive(PY)]
enum Variant {
    Name(VariantName),
}

mod same_path {
    use py_rs::PY;

    #[derive(PY)]
    #[py(export_to = "shared/Model.py")]
    pub struct First;

    #[derive(PY)]
    #[py(export_to = "shared/Model.py")]
    pub struct Second;
}

#[derive(PY)]
struct Both {
    first: same_path::First,
    second: same_path::Second,
}

#[test]
fn generated_declarations_and_dependencies() {
    let text = User::export_to_string().unwrap();
    assert!(text.contains("class User:"));
    assert!(text.contains("display_name: str"));
    assert!(text.contains("address: Address | None"));
    assert!(text.contains("from .models.Address import Address"));
    assert!(!text.contains("secret:"));
    assert_eq!(User::dependencies().len(), 3);
    assert!(Status::decl().contains("OnHold = \"on-hold\""));
    assert!(Status::decl().contains("Quoted = \"quoted\\\"\\\\\\u0001\""));
    assert!(Event::decl().contains("Event: TypeAlias = EventCreated | EventMoved | EventDeleted"));
    assert_eq!(Left::dependencies().len(), 1);
    assert_eq!(Right::dependencies().len(), 1);
    assert!(ZeroArray::decl().contains("empty: tuple[()]"));
    assert!(RawVariant::decl().contains("type = \"type\""));
}

#[test]
fn invalid_paths_and_duplicate_python_names_are_rejected() {
    assert!(matches!(
        BadPath::export_to_string(),
        Err(ExportError::InvalidPath(_))
    ));
    assert!(
        matches!(Ambiguous::export_to_string(), Err(ExportError::NameCollision(name)) if name == "Thing")
    );
    assert!(
        matches!(Variant::export_to_string(), Err(ExportError::NameCollision(name)) if name == "VariantName")
    );
    assert!(matches!(
        Both::export_all(),
        Err(ExportError::ConflictingFile(_))
    ));
}

#[test]
fn export_cyclic_dependencies() {
    Left::export_all().unwrap();
}
