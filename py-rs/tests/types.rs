#![allow(dead_code)]

use py_rs::{ExportError, PY};
use std::borrow::{Borrow, Cow};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::net::{IpAddr, SocketAddr};
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(PY, Clone)]
struct Nested {
    id: NonZeroU64,
}

#[derive(PY)]
#[py(frozen)]
struct HashableKey {
    id: u64,
}

#[derive(PY)]
struct InvalidSet {
    members: HashSet<Nested>,
}

#[derive(PY)]
struct ValidMap {
    members: HashMap<HashableKey, String>,
}

#[derive(PY)]
struct InvalidMap {
    members: HashMap<Vec<u8>, String>,
}

#[derive(PY)]
struct InvalidNestedSet {
    groups: Vec<HashSet<Nested>>,
}

#[derive(PY)]
struct ValidSet {
    members: HashSet<HashableKey>,
}

#[derive(PY)]
#[py(frozen)]
struct FrozenWithList {
    values: Vec<u8>,
}

#[derive(PY)]
#[py(frozen)]
struct RecursiveKey {
    next: Option<Box<RecursiveKey>>,
}

#[derive(PY)]
#[py(export)]
struct TypeCoverage {
    queue: VecDeque<Arc<Nested>>,
    linked: LinkedList<bool>,
    #[py(type = "list[int]")]
    heap: BinaryHeap<u8>,
    ordered_set: BTreeSet<String>,
    hashed_set: HashSet<u16>,
    ordered_map: BTreeMap<String, Nested>,
    hashed_map: HashMap<String, Nested>,
    borrowed: &'static Nested,
    cow: Cow<'static, str>,
    cow_slice: Cow<'static, [u8]>,
    cow_nested: Cow<'static, [Nested]>,
    path: PathBuf,
    ip: IpAddr,
    #[py(type = "tuple[str, int]")]
    socket: SocketAddr,
    #[py(type = "decimal.Decimal", import = "decimal")]
    amount: u64,
    one: (u8,),
    three: (u8, String, Nested),
    array: [Nested; 2],
    long_array: [u8; 13],
}

#[test]
fn standard_types_render_and_preserve_dependencies() {
    let text = TypeCoverage::export_to_string().unwrap();
    for annotation in [
        "queue: collections.deque[Nested]",
        "linked: list[bool]",
        "heap: list[int]",
        "ordered_set: set[str]",
        "hashed_set: set[int]",
        "ordered_map: dict[str, Nested]",
        "hashed_map: dict[str, Nested]",
        "borrowed: Nested",
        "cow: str",
        "cow_slice: list[int]",
        "cow_nested: list[Nested]",
        "path: pathlib.Path",
        "ip: ipaddress.IPv4Address | ipaddress.IPv6Address",
        "socket: tuple[str, int]",
        "amount: decimal.Decimal",
        "one: tuple[int]",
        "three: tuple[int, str, Nested]",
        "array: tuple[Nested, Nested]",
        "long_array: tuple[int, ...]",
        "from .Nested import Nested",
        "import collections",
        "import pathlib",
        "import ipaddress",
        "import decimal",
    ] {
        assert!(text.contains(annotation), "missing {annotation}: {text}");
    }
    assert_eq!(TypeCoverage::dependencies().len(), 7);
}

struct BorrowedShape;
struct OwnedShape(BorrowedShape);

impl Borrow<BorrowedShape> for OwnedShape {
    fn borrow(&self) -> &BorrowedShape {
        &self.0
    }
}

impl ToOwned for BorrowedShape {
    type Owned = OwnedShape;
    fn to_owned(&self) -> OwnedShape {
        OwnedShape(BorrowedShape)
    }
}

impl PY for BorrowedShape {
    fn name() -> String {
        "str".into()
    }
    fn inline() -> String {
        "str".into()
    }
}

impl PY for OwnedShape {
    fn name() -> String {
        "bytes".into()
    }
    fn inline() -> String {
        "bytes".into()
    }
}

#[test]
fn cow_uses_borrowed_type_mapping() {
    assert_eq!(Cow::<'static, BorrowedShape>::inline(), "str");
    assert_ne!(
        Cow::<'static, BorrowedShape>::inline(),
        OwnedShape::inline()
    );
}

#[test]
fn set_and_map_keys_must_be_python_hashable() {
    assert!(matches!(
        InvalidSet::export_to_string(),
        Err(ExportError::UnhashableType(name)) if name == "Nested"
    ));
    assert!(ValidMap::export_to_string().is_ok());
    assert!(ValidSet::export_to_string().is_ok());
    assert!(matches!(
        InvalidMap::export_to_string(),
        Err(ExportError::UnhashableType(name)) if name == "list[int]"
    ));
    assert!(matches!(
        InvalidNestedSet::export_to_string(),
        Err(ExportError::UnhashableType(name)) if name == "Nested"
    ));
    assert!(!FrozenWithList::is_hashable());
    assert!(!RecursiveKey::is_hashable());
}

#[cfg(feature = "bytes-impl")]
#[test]
fn bytes_feature_maps_to_python_bytes() {
    assert_eq!(bytes::Bytes::inline(), "bytes");
    assert_eq!(bytes::BytesMut::inline(), "bytearray");
}

#[cfg(feature = "chrono-impl")]
#[test]
fn chrono_feature_maps_date_types() {
    assert_eq!(chrono::NaiveDate::inline(), "datetime.date");
    assert_eq!(
        chrono::DateTime::<chrono::Utc>::inline(),
        "datetime.datetime"
    );
}

#[cfg(feature = "serde-json-impl")]
#[derive(PY)]
struct JsonPayload {
    value: serde_json::Value,
    map: serde_json::Map<String, serde_json::Value>,
}

#[cfg(feature = "serde-json-impl")]
#[test]
fn serde_json_feature_maps_dynamic_values() {
    assert_eq!(serde_json::Value::inline(), "_PyRsJsonValue");
    assert_eq!(serde_json::Number::inline(), "int | float");
    assert_eq!(
        serde_json::Map::<String, serde_json::Value>::inline(),
        "dict[str, _PyRsJsonValue]"
    );
    let text = JsonPayload::export_to_string().unwrap();
    assert!(text.contains("from typing import TypeAlias"));
    assert!(text.contains("_PyRsJsonValue: TypeAlias = None | bool | int | float | str"));
    assert!(text.contains("value: _PyRsJsonValue"));
    JsonPayload::export_all().unwrap();
}

#[cfg(feature = "url-impl")]
#[test]
fn url_feature_maps_url() {
    assert_eq!(url::Url::inline(), "str");
}

#[cfg(feature = "uuid-impl")]
#[test]
fn uuid_feature_maps_uuid() {
    assert_eq!(uuid::Uuid::inline(), "uuid.UUID");
}
