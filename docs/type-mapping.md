# Rust to Python type mapping

These mappings select field annotations and referenced Python objects inside generated dataclasses. They do not convert Rust values to Python values or validate constructor arguments at runtime.

## Built-in mappings

| Rust type | Python annotation |
| --- | --- |
| `bool` | `bool` |
| Signed and unsigned integers, including `isize` and `usize` | `int` |
| `NonZero*` integers | `int` |
| `f32`, `f64` | `float` |
| `String`, `str`, `&str`, `char` | `str` |
| `()` | `None` |
| `Path`, `PathBuf`, `&Path` | `pathlib.Path` |
| `Ipv4Addr`, `Ipv6Addr` | `ipaddress.IPv4Address`, `ipaddress.IPv6Address` |
| `IpAddr` | `ipaddress.IPv4Address | ipaddress.IPv6Address` |
| `SocketAddr`, `SocketAddrV4`, `SocketAddrV6` | `str` |
| `Duration` | `datetime.timedelta` |
| `SystemTime` | `datetime.datetime` |
| `Option<T>` | `T | None` |
| `Vec<T>`, `LinkedList<T>`, `BinaryHeap<T>`, `[T]`, `&[T]` | `list[T]` |
| `VecDeque<T>` | `collections.deque[T]` |
| `HashSet<T, S>`, `BTreeSet<T>` | `set[T]` |
| `HashMap<K, V, S>`, `BTreeMap<K, V>` | `dict[K, V]` |
| `[T; 0]` | `tuple[()]` |
| `[T; N]` for `1 <= N <= 12` | Fixed-length tuple with `T` repeated `N` times |
| `[T; N]` for `N > 12` | `tuple[T, ...]`; the exact length is not represented |
| Tuples with 1–10 elements | Fixed-length tuple with each element type shown |
| `Box<T>`, `Rc<T>`, `Arc<T>`, `Cell<T>`, `RefCell<T>`, `Mutex<T>`, `RwLock<T>`, `&T` | `T` |
| `Rc::Weak<T>`, `Arc::Weak<T>` | `T | None` |
| `Cow<'a, T>` | The mapping of borrowed `T`; ownership is not represented |

For example, `Cow<'a, str>` maps to `str`, and `Cow<'a, [u8]>` maps to `list[int]`. `Option<T>` permits `None` but does **not** make a dataclass constructor argument optional. A Rust `u8` or `NonZeroU8` maps to `int` without range or nonzero enforcement; `char` maps to `str` without a one-character check.

Python sets require hashable elements and dictionaries require hashable keys. The exporter checks the mapped type's hashability, including frozen dataclasses and their fields, before writing these declarations.

## Cargo feature mappings

| Feature | Rust type | Python annotation |
| --- | --- | --- |
| `bytes-impl` | `bytes::Bytes`, `bytes::BytesMut` | `bytes`, `bytearray` |
| `chrono-impl` | `chrono::NaiveDate`, `NaiveTime`, `NaiveDateTime` | `datetime.date`, `datetime.time`, `datetime.datetime` |
| `chrono-impl` | `chrono::DateTime<Tz>` | `datetime.datetime` |
| `bigdecimal-impl` | `bigdecimal::BigDecimal` | `decimal.Decimal` |
| `bson-impl` | `bson::oid::ObjectId`, `bson::Uuid` | `str`, `uuid.UUID` |
| `heapless-impl` | `heapless::Vec<T, N>`, `Deque<T, N>`, `String<N>` | `list[T]`, `list[T]`, `str` |
| `indexmap-impl` | `indexmap::IndexMap<K, V>`, `IndexSet<T>` | `dict[K, V]`, `list[T]` |
| `ordered-float-impl` | `ordered_float::OrderedFloat<T>`, `NotNan<T>` | `T` |
| `semver-impl` | `semver::Version`, `VersionReq` | `str` |
| `smol-str-impl` | `smol_str::SmolStr` | `str` |
| `tokio-impl` | `tokio::sync::Mutex<T>`, `RwLock<T>`, `OnceCell<T>` | `T`, `T`, `T | None` |
| `serde-json-impl` | `serde_json::Value` | Recursive `_PyRsJsonValue` type alias covering null, booleans, numbers, strings, arrays, and string-keyed objects |
| `serde-json-impl` | `serde_json::Number`, `Map<String, Value>` | `int | float`, `dict[str, _PyRsJsonValue]` |
| `url-impl` | `url::Url` | `str` |
| `uuid-impl` | `uuid::Uuid` | `uuid.UUID` |

Enable a feature in `Cargo.toml`, for example `rust-py-models = { path = "path/to/rust-py-models/rust-py-models", features = ["chrono-impl"] }`. `url::Url` is represented as a Python string because the standard library has no equivalent URL value class; the annotation does not parse URLs.

Temporal types map directly to the closest Python standard-library type. Python's `datetime.datetime` does not encode the Rust timezone parameter in its annotation, and the generated model does not validate `tzinfo`, offsets, sign, or precision.

## Unmapped Rust types

`Result<T, E>` and range types have no automatic Python mapping because they need an application-specific object representation. A field can use `#[py(unsafe_type = "...")]` as an explicit unchecked annotation override, but doing so does not generate Python behavior for that Rust type, rewrite imported names inside the raw annotation, or export a referenced model automatically. Use `#[py(as = "RustType")]` when another Rust type already has the desired mapping and dependency behavior.

The generator produces concrete Python classes for derived structs and enums, but its field annotations remain descriptive. JSON serialization, Rust FFI, and automatic value conversion are outside the current implementation.
