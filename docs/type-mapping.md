# Rust to Python type mapping

Use this reference to determine the annotation emitted for a Rust field. Mappings describe Python objects; they do not convert Rust values or validate Python constructor arguments.

## Standard-library mappings

| Rust type | Python annotation |
| --- | --- |
| `bool` | `bool` |
| Signed and unsigned integers, including `isize`, `usize`, and `NonZero*` | `int` |
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
| `Result<T, E>` | `T` |
| `Vec<T>`, `LinkedList<T>`, `BinaryHeap<T>`, `[T]`, `&[T]` | `list[T]` |
| `VecDeque<T>` | `collections.deque[T]` |
| `HashSet<T, S>`, `BTreeSet<T>` | `set[T]` |
| `HashMap<K, V, S>`, `BTreeMap<K, V>` | `dict[K, V]` |
| `[T; 0]` | `tuple[()]` |
| `[T; N]`, where `1 <= N <= 12` | Fixed-length tuple containing `N` instances of `T` |
| `[T; N]`, where `N > 12` | `tuple[T, ...]` |
| Tuples with 1–10 elements | Fixed-length tuple preserving each element type |
| `Box<T>`, `Rc<T>`, `Arc<T>`, `Cell<T>`, `RefCell<T>`, `Mutex<T>`, `RwLock<T>`, `&T` | `T` |
| `Rc::Weak<T>`, `Arc::Weak<T>` | `T | None` |
| `Cow<'a, T>` | The mapping for borrowed `T` |

Python annotations do not carry these Rust constraints:

- `Option<T>` accepts `None` but does not make a dataclass argument optional.
- `Result<T, E>` emits the annotation for `T`; the annotation does not represent `E` or a Python `Result` object.
- integer mappings do not enforce Rust ranges or nonzero constraints;
- `char` does not enforce a one-character string;
- arrays longer than 12 elements do not retain their exact length;
- ownership, borrowing, locking, and `Cow` ownership state are not represented.

The exporter rejects a set element or dictionary key when its generated Python type is known to be unhashable. A derived dataclass is hashable only when its dataclass options and fields make it hashable.

When a generic parameter appears only as the `E` of `Result<T, E>`, the generated Python model omits that parameter. Other uses of the parameter retain it.

## Optional crate mappings

Enable the corresponding Cargo feature on `rust-py-models`:

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
| `serde-json-impl` | `serde_json::Value` | Recursive union of JSON-compatible Python values |
| `serde-json-impl` | `serde_json::Number`, `Map<String, Value>` | `int | float`, `dict[str, JSON value]` |
| `smol-str-impl` | `smol_str::SmolStr` | `str` |
| `tokio-impl` | `tokio::sync::Mutex<T>`, `RwLock<T>`, `OnceCell<T>` | `T`, `T`, `T | None` |
| `url-impl` | `url::Url` | `str` |
| `uuid-impl` | `uuid::Uuid` | `uuid.UUID` |

For example:

```toml
[dependencies]
rust-py-models = { version = "0.1.1", features = ["chrono-impl", "uuid-impl"] }
```

`DateTime<Tz>` maps to `datetime.datetime`; the annotation does not retain the Rust timezone parameter or validate `tzinfo`. `url::Url` maps to `str` and does not parse URLs.

## Types without a default mapping

Range types require an application-specific representation and therefore have no automatic mapping.

Use `#[py(as = "RustType")]` to reuse an existing mapping. Use `#[py(unsafe_type = "...")]` only when you can supply any required Python imports and dependencies and accept that the annotation is unchecked. See [generation and export](generation.md#configure-generated-declarations) for these attributes.
