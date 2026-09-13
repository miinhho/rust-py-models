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
| `Option<T>` | `T | None` |
| `Vec<T>`, `LinkedList<T>`, `[T]`, `&[T]` | `list[T]` |
| `VecDeque<T>` | `collections.deque[T]` |
| `HashSet<T, S>`, `BTreeSet<T>` | `set[T]` |
| `HashMap<K, V, S>`, `BTreeMap<K, V>` | `dict[K, V]` |
| `[T; 0]` | `tuple[()]` |
| `[T; N]` for `1 <= N <= 12` | Fixed-length tuple with `T` repeated `N` times |
| `[T; N]` for `N > 12` | `tuple[T, ...]`; the exact length is not represented |
| Tuples with 1–10 elements | Fixed-length tuple with each element type shown |
| `Box<T>`, `Rc<T>`, `Arc<T>`, `&T` | `T` |
| `Cow<'a, T>` | The mapping of borrowed `T`; ownership is not represented |

For example, `Cow<'a, str>` maps to `str`, and `Cow<'a, [u8]>` maps to `list[int]`. `Option<T>` permits `None` but does **not** make a dataclass constructor argument optional. A Rust `u8` or `NonZeroU8` maps to `int` without range or nonzero enforcement; `char` maps to `str` without a one-character check.

Python sets require hashable elements and dictionaries require hashable keys. The exporter checks the mapped type's hashability, including frozen dataclasses and their fields, before writing these declarations.

## Cargo feature mappings

| Feature | Rust type | Python annotation |
| --- | --- | --- |
| `bytes-impl` | `bytes::Bytes`, `bytes::BytesMut` | `bytes`, `bytearray` |
| `chrono-impl` | `chrono::NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime<Tz>` | `datetime.date`, `datetime.time`, `datetime.datetime`, `datetime.datetime` |
| `serde-json-impl` | `serde_json::Value` | Recursive `_PyRsJsonValue` type alias covering null, booleans, numbers, strings, arrays, and string-keyed objects |
| `serde-json-impl` | `serde_json::Number`, `Map<String, Value>` | `int | float`, `dict[str, _PyRsJsonValue]` |
| `url-impl` | `url::Url` | `str` |
| `uuid-impl` | `uuid::Uuid` | `uuid.UUID` |

Enable a feature in `Cargo.toml`, for example `py-rs = { path = "path/to/py-rs/py-rs", features = ["chrono-impl"] }`. `url::Url` is represented as a Python string because the standard library has no equivalent URL value class; the annotation does not parse URLs.

## Unmapped Rust types

`Result<T, E>`, `BinaryHeap`, `Weak`, `Cell`, `RefCell`, `Mutex`, `RwLock`, and `SocketAddr` have no automatic Python mapping. `Result` in particular needs an application-specific object representation. A field can use `#[py(type = "...")]` as an explicit annotation override, but doing so does not generate Python behavior for that Rust type or export a referenced model automatically.

The generator produces concrete Python classes for derived structs and enums, but its field annotations remain descriptive. JSON serialization, Rust FFI, and automatic value conversion are outside the current implementation.
