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
struct CycleA {
    b: Option<Box<CycleB>>,
}

#[derive(PY)]
struct CycleB {
    c: Option<Box<CycleC>>,
}

#[derive(PY)]
struct CycleC {
    a: Option<Box<CycleA>>,
}

#[derive(PY)]
#[py(export)]
struct ZeroArray {
    empty: [u8; 0],
}

#[derive(PY)]
#[py(export, frozen, slots = true, kw_only)]
struct ImmutableUser {
    id: u64,
    name: String,
}

#[derive(PY)]
#[py(export, frozen, slots, kw_only)]
enum AuditEvent {
    Created {
        id: u64,
    },
    #[py(frozen = false, slots = false, kw_only = false)]
    Mutable(String),
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

#[derive(PY)]
#[py(rename = "decimal")]
struct CollidingImport {
    #[py(type = "decimal.Decimal", import = "decimal")]
    amount: u64,
}

#[test]
fn generated_declarations_and_dependencies() {
    let text = User::export_to_string().unwrap();
    assert!(text.contains("class User:"));
    assert!(text.contains("display_name: str"));
    assert!(text.contains("address: Address | None"));
    assert!(text.contains("from .models.Address import Address"));
    assert!(text.find("from .models.Address import Address") < text.find("@dataclass"));
    let cyclic = Left::export_to_string().unwrap();
    assert!(cyclic.contains("from .Right import Right  # noqa: E402 - cyclic dependency"));
    assert!(cyclic.find("from .Right import Right") > cyclic.find("class Left:"));
    assert!(!text.contains("secret:"));
    assert_eq!(User::dependencies().len(), 3);
    assert!(Status::decl().contains("OnHold = \"on-hold\""));
    assert!(Status::decl().contains("Quoted = \"quoted\\\"\\\\\\u0001\""));
    assert!(Event::decl().contains("Event: TypeAlias = EventCreated | EventMoved | EventDeleted"));
    assert_eq!(Left::dependencies().len(), 1);
    assert_eq!(Right::dependencies().len(), 1);
    assert!(ZeroArray::decl().contains("empty: tuple[()]"));
    assert!(ImmutableUser::decl().contains("@dataclass(frozen=True, slots=True, kw_only=True)"));
    let audit = AuditEvent::decl();
    assert!(audit
        .contains("@dataclass(frozen=True, slots=True, kw_only=True)\nclass AuditEventCreated"));
    assert!(audit
        .contains("@dataclass(frozen=False, slots=False, kw_only=False)\nclass AuditEventMutable"));
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
    assert!(
        matches!(CollidingImport::export_to_string(), Err(ExportError::NameCollision(name)) if name == "decimal")
    );
}

#[test]
fn export_cyclic_dependencies() {
    Left::export_all().unwrap();
    CycleA::export_all().unwrap();
    assert!(CycleA::export_to_string()
        .unwrap()
        .contains("from .CycleB import CycleB  # noqa: E402 - cyclic dependency"));
}
