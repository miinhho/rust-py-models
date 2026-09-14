#![allow(dead_code)]
#![allow(deprecated)]

use rust_py_models::{ExportError, PY};
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

mod nested_cycle {
    use rust_py_models::PY;

    #[derive(PY)]
    #[py(export, export_to = "graph/left/NestedCycleA.py")]
    pub struct NestedCycleA {
        pub b: Option<Box<NestedCycleB>>,
    }

    #[derive(PY)]
    #[py(export_to = "graph/right/NestedCycleB.py")]
    pub struct NestedCycleB {
        pub c: Option<Box<NestedCycleC>>,
    }

    #[derive(PY)]
    #[py(export_to = "graph/NestedCycleC.py")]
    pub struct NestedCycleC {
        pub a: Option<Box<NestedCycleA>>,
    }
}

/// Legacy user model.
///
/// Retained for migration.
#[deprecated(since = "0.1.0", note = "Use User")]
#[derive(PY)]
#[py(export)]
struct LegacyModel {
    /// Stable identifier.
    id: u64,
}

/// Documented status values.
#[derive(PY)]
#[py(export)]
enum DocumentedStatus {
    /// Work is ready.
    Ready,
}

/// Documented event variants.
#[derive(PY)]
#[py(export)]
enum DocumentedEvent {
    /// An item was created.
    Created {
        /// Created item identifier.
        id: u64,
    },
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
    use rust_py_models::PY;
    #[derive(PY)]
    #[py(export_to = "first/Thing.py")]
    pub struct Thing;
}

mod second {
    use rust_py_models::PY;
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
    use rust_py_models::PY;

    #[derive(PY)]
    #[py(export_to = "shared/Model.py")]
    pub struct First;

    #[derive(PY)]
    #[py(export_to = "shared/Model.py")]
    pub struct Second;
}

#[derive(PY)]
#[py(export)]
struct Both {
    first: same_path::First,
    second: same_path::Second,
}

#[derive(PY)]
struct CollidingImport {
    #[py(rename = "pathlib")]
    path: std::path::PathBuf,
}

#[derive(PY)]
struct ImportedModel {
    value: u64,
}

#[derive(PY)]
#[py(export)]
struct ImportedModelShadow {
    #[py(rename = "ImportedModel")]
    imported: ImportedModel,
}

mod portable_paths {
    use rust_py_models::PY;

    #[derive(PY)]
    #[py(export_to = "models/Thing.py")]
    pub struct Upper;

    #[derive(PY)]
    #[py(export_to = "Models/thing.py")]
    pub struct Lower;
}

#[derive(PY)]
struct PortablePathCollision {
    upper: portable_paths::Upper,
    lower: portable_paths::Lower,
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
    let status = Status::export_to_string().unwrap();
    assert!(status.contains("OnHold = \"on-hold\""));
    assert!(status.contains("Quoted = 'quoted\"\\\\\\u0001'"));
    assert!(Event::export_to_string()
        .unwrap()
        .contains("Event: TypeAlias = EventCreated | EventMoved | EventDeleted"));
    assert!(ZeroArray::export_to_string()
        .unwrap()
        .contains("empty: tuple[()]"));
    assert!(ImmutableUser::export_to_string()
        .unwrap()
        .contains("@dataclass(frozen=True, slots=True, kw_only=True)"));
    let audit = AuditEvent::export_to_string().unwrap();
    assert!(audit
        .contains("@dataclass(frozen=True, slots=True, kw_only=True)\nclass AuditEventCreated"));
    assert!(audit
        .contains("@dataclass(frozen=False, slots=False, kw_only=False)\nclass AuditEventMutable"));
    assert!(RawVariant::export_to_string()
        .unwrap()
        .contains("type = \"type\""));
}

#[test]
fn rust_documentation_and_deprecation_are_rendered() {
    let legacy = LegacyModel::export_to_string().unwrap();
    assert!(legacy.contains("\"Legacy user model.\\n\\nRetained for migration.\\n\\nDeprecated: Use User (since 0.1.0).\""), "{legacy}");
    assert!(
        legacy.contains("id: int\n    \"Stable identifier.\""),
        "{legacy}"
    );

    let status = DocumentedStatus::export_to_string().unwrap();
    assert!(status.contains("\"Documented status values.\""), "{status}");
    assert!(
        status.contains("Ready = \"Ready\"\n    \"Work is ready.\""),
        "{status}"
    );

    let event = DocumentedEvent::export_to_string().unwrap();
    assert!(event.contains("\"An item was created.\""), "{event}");
    assert!(
        event.contains("id: int\n    \"Created item identifier.\""),
        "{event}"
    );
    assert!(event.contains("# Documented event variants."), "{event}");
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
        PortablePathCollision::export_to_string(),
        Err(ExportError::PortablePathCollision { .. })
    ));
    let colliding = CollidingImport::export_to_string().unwrap();
    assert!(
        colliding.contains("import pathlib as _py_rs_pathlib"),
        "{colliding}"
    );
    assert!(colliding.contains("pathlib: _py_rs_pathlib.Path"));
    let model_shadow = ImportedModelShadow::export_to_string().unwrap();
    assert!(
        model_shadow.contains("from .ImportedModel import ImportedModel as _py_rs_ImportedModel"),
        "{model_shadow}"
    );
    assert!(model_shadow.contains("ImportedModel: _py_rs_ImportedModel"));
}

#[test]
fn combines_multiple_declarations_in_one_module() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock before Unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "rust-py-models-shared-module-{}-{nonce}",
        std::process::id()
    ));

    Both::export_all_to(&dir).unwrap();
    let shared = std::fs::read_to_string(dir.join("shared/Model.py")).unwrap();
    let owner = module_path!();
    assert!(
        shared.contains(&format!(
            "# Rust types: {owner}::same_path::First, {owner}::same_path::Second"
        )),
        "{shared}"
    );
    assert!(shared.contains("class First:"), "{shared}");
    assert!(shared.contains("class Second:"), "{shared}");
    assert!(!shared.contains("from .Model import"), "{shared}");
    let both = std::fs::read_to_string(dir.join("Both.py")).unwrap();
    assert!(
        both.contains("from .shared.Model import First, Second"),
        "{both}"
    );
    Both::export_all_to(&dir).unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("shared/Model.py")).unwrap(),
        shared
    );

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn export_cyclic_dependencies() {
    Left::export_all().unwrap();
    CycleA::export_all().unwrap();
    assert!(CycleA::export_to_string()
        .unwrap()
        .contains("from .CycleB import CycleB  # noqa: E402 - cyclic dependency"));
    let nested = nested_cycle::NestedCycleA::export_to_string().unwrap();
    assert!(
        nested.contains(
            "from ..right.NestedCycleB import NestedCycleB  # noqa: E402 - cyclic dependency"
        ),
        "{nested}"
    );
}

#[test]
fn exports_dependency_graph_to_explicit_directory() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock before Unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "rust-py-models-export-all-to-{}-{nonce}",
        std::process::id()
    ));

    User::export_all_to(&dir).unwrap();
    assert!(dir.join("__init__.py").is_file());
    assert!(dir.join("User.py").is_file());
    assert!(dir.join("models/Address.py").is_file());
    assert!(dir.join("Status.py").is_file());
    assert!(dir.join("Event.py").is_file());

    std::fs::remove_dir_all(dir).unwrap();
}
