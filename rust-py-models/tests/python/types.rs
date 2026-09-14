#![allow(dead_code)]

#[cfg(test)]
use rust_py_models::ExportError;
use rust_py_models::PY;
use std::borrow::{Borrow, Cow};
#[cfg(test)]
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::net::{IpAddr, SocketAddr};
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(test)]
use std::sync::{Mutex, RwLock, Weak};
use std::time::{Duration, SystemTime};

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

struct UnmappedError;

#[derive(PY)]
#[py(export)]
struct ResultCoverage {
    item: Result<Nested, UnmappedError>,
    items: Vec<Result<Nested, UnmappedError>>,
    optional: Option<Result<String, UnmappedError>>,
    #[py(as = "String")]
    overridden: Result<u64, UnmappedError>,
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
    socket: SocketAddr,
    #[py(unsafe_type = "decimal.Decimal", import = "decimal")]
    amount: u64,
    one: (u8,),
    three: (u8, String, Nested),
    array: [Nested; 2],
    long_array: [u8; 13],
}

#[derive(PY)]
#[py(export)]
struct FeatureCoverage {
    duration: Duration,
    system_time: SystemTime,
    #[cfg(feature = "bytes-impl")]
    immutable_bytes: bytes::Bytes,
    #[cfg(feature = "bytes-impl")]
    mutable_bytes: bytes::BytesMut,
    #[cfg(feature = "chrono-impl")]
    date: chrono::NaiveDate,
    #[cfg(feature = "chrono-impl")]
    time: chrono::NaiveTime,
    #[cfg(feature = "chrono-impl")]
    naive_datetime: chrono::NaiveDateTime,
    #[cfg(feature = "chrono-impl")]
    utc_datetime: chrono::DateTime<chrono::Utc>,
    #[cfg(feature = "chrono-impl")]
    fixed_datetime: chrono::DateTime<chrono::FixedOffset>,
    #[cfg(feature = "chrono-impl")]
    timezone_datetime: chrono::DateTime<chrono_tz::Tz>,
    #[cfg(feature = "serde-json-impl")]
    json: serde_json::Value,
    #[cfg(feature = "url-impl")]
    url: url::Url,
    #[cfg(feature = "uuid-impl")]
    uuid: uuid::Uuid,
    #[cfg(feature = "bigdecimal-impl")]
    decimal: bigdecimal::BigDecimal,
    #[cfg(feature = "bson-impl")]
    object_id: bson::oid::ObjectId,
    #[cfg(feature = "bson-impl")]
    bson_uuid: bson::Uuid,
    #[cfg(feature = "indexmap-impl")]
    index_map: indexmap::IndexMap<String, u64>,
    #[cfg(feature = "indexmap-impl")]
    index_set: indexmap::IndexSet<String>,
    #[cfg(feature = "ordered-float-impl")]
    ordered: ordered_float::OrderedFloat<f64>,
    #[cfg(feature = "ordered-float-impl")]
    not_nan: ordered_float::NotNan<f32>,
    #[cfg(feature = "heapless-impl")]
    bounded_vec: heapless::Vec<u64, 8>,
    #[cfg(feature = "heapless-impl")]
    bounded_deque: heapless::Deque<String, 8>,
    #[cfg(feature = "heapless-impl")]
    bounded_string: heapless::String<32>,
    #[cfg(feature = "semver-impl")]
    version: semver::Version,
    #[cfg(feature = "semver-impl")]
    version_requirement: semver::VersionReq,
    #[cfg(feature = "smol-str-impl")]
    small_string: smol_str::SmolStr,
    #[cfg(feature = "tokio-impl")]
    async_mutex: tokio::sync::Mutex<u64>,
    #[cfg(feature = "tokio-impl")]
    async_rw_lock: tokio::sync::RwLock<String>,
    #[cfg(feature = "tokio-impl")]
    async_once: tokio::sync::OnceCell<u64>,
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
        "socket: str",
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
}

#[test]
fn additional_standard_types_use_serialized_python_shapes() {
    assert_eq!(BinaryHeap::<u64>::inline(), "list[int]");
    assert_eq!(SocketAddr::inline(), "str");
    assert_eq!(Duration::inline(), "datetime.timedelta");
    assert_eq!(SystemTime::inline(), "datetime.datetime");
    assert_eq!(Cell::<u64>::inline(), "int");
    assert_eq!(RefCell::<String>::inline(), "str");
    assert_eq!(Mutex::<u64>::inline(), "int");
    assert_eq!(RwLock::<String>::inline(), "str");
    assert_eq!(Weak::<String>::inline(), "str | None");
    assert_eq!(Result::<Nested, UnmappedError>::inline(), "Nested");
    assert_eq!(
        Vec::<Result<Nested, UnmappedError>>::inline(),
        "list[Nested]"
    );
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
    fn type_spec() -> rust_py_models::TypeSpec {
        rust_py_models::TypeSpec::named("str")
    }
}

impl PY for OwnedShape {
    fn type_spec() -> rust_py_models::TypeSpec {
        rust_py_models::TypeSpec::named("bytes")
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
fn chrono_feature_maps_temporal_types() {
    assert_eq!(chrono::NaiveDate::inline(), "datetime.date");
    assert_eq!(chrono::NaiveTime::inline(), "datetime.time");
    assert_eq!(chrono::NaiveDateTime::inline(), "datetime.datetime");
    assert_eq!(
        chrono::DateTime::<chrono::Utc>::inline(),
        "datetime.datetime"
    );
    assert_eq!(
        chrono::DateTime::<chrono::FixedOffset>::inline(),
        "datetime.datetime"
    );
    assert_eq!(
        chrono::DateTime::<chrono_tz::Tz>::inline(),
        "datetime.datetime"
    );
}

#[cfg(feature = "serde-json-impl")]
#[derive(PY)]
#[py(export)]
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
    assert!(text.contains("value: _PyRsJsonValue"));
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

#[cfg(feature = "bigdecimal-impl")]
#[test]
fn bigdecimal_feature_maps_decimal() {
    assert_eq!(bigdecimal::BigDecimal::inline(), "decimal.Decimal");
}

#[cfg(feature = "bson-impl")]
#[test]
fn bson_feature_maps_identifiers() {
    assert_eq!(bson::oid::ObjectId::inline(), "str");
    assert_eq!(bson::Uuid::inline(), "uuid.UUID");
}

#[cfg(feature = "indexmap-impl")]
#[test]
fn indexmap_feature_preserves_collection_shapes() {
    assert_eq!(
        indexmap::IndexMap::<String, u64>::inline(),
        "dict[str, int]"
    );
    assert_eq!(indexmap::IndexSet::<String>::inline(), "list[str]");
}

#[cfg(feature = "ordered-float-impl")]
#[test]
fn ordered_float_feature_maps_numbers() {
    assert_eq!(ordered_float::OrderedFloat::<f64>::inline(), "float");
    assert_eq!(ordered_float::NotNan::<f32>::inline(), "float");
}

#[cfg(feature = "heapless-impl")]
#[test]
fn heapless_feature_maps_bounded_collections() {
    assert_eq!(heapless::Vec::<u64, 8>::inline(), "list[int]");
    assert_eq!(heapless::Deque::<String, 8>::inline(), "list[str]");
    assert_eq!(heapless::String::<32>::inline(), "str");
}

#[cfg(feature = "semver-impl")]
#[test]
fn semver_feature_maps_version_strings() {
    assert_eq!(semver::Version::inline(), "str");
    assert_eq!(semver::VersionReq::inline(), "str");
}

#[cfg(feature = "smol-str-impl")]
#[test]
fn smol_str_feature_maps_strings() {
    assert_eq!(smol_str::SmolStr::inline(), "str");
}

#[cfg(feature = "tokio-impl")]
#[test]
fn tokio_feature_maps_synchronization_wrappers() {
    assert_eq!(tokio::sync::Mutex::<u64>::inline(), "int");
    assert_eq!(tokio::sync::RwLock::<String>::inline(), "str");
    assert_eq!(tokio::sync::OnceCell::<u64>::inline(), "int | None");
}
