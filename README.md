# py-rs

`py-rs` generates Python 3.10+ dataclass and Enum models from Rust structs and enums. Its derive/trait/export flow follows the [ts-rs project](https://github.com/Aleph-Alpha/ts-rs): the proc macro reads Rust syntax at compile time, the `PY` trait describes the Python representation, and `cargo test` can write bindings.

```toml
[dependencies]
py-rs = { path = "path/to/py-rs/py-rs" }
```

```rust
use py_rs::PY;

#[derive(PY)]
struct Address { city: String }

#[derive(PY)]
#[py(export)]
struct User { id: u64, address: Option<Address> }
```

`cargo test export_bindings` generates `bindings/User.py` and `bindings/Address.py` in the package running the test. `#[py(export)]` adds a test that calls `PY::export_all()`. You can call `User::export()` for one type, `User::export_all()` for its dependency graph, or `User::export_to_string()` to inspect the generated source. `PY_RS_EXPORT_DIR` overrides the default `bindings` directory. `#[py(export_to = "models/User.py")]` sets a path relative to that directory.

```sh
PY_RS_EXPORT_DIR=python/bindings cargo test export_bindings
```

Supported type mappings include integer → `int`, float → `float`, `String`/`char` → `str`, `bool` → `bool`, `Option<T>` → `T | None`, `Vec<T>` → `list[T]`, maps → `dict[K, V]`, sets → `set[T]`, `Box<T>` → `T`, arrays and 2-tuples → `tuple[...]`. Unit enums become `str, Enum` classes. Enums with payloads become one dataclass per variant and a `TypeAlias` union. Named and tuple structs become dataclasses. `#[py(rename = "...")]` changes a Python class or field name, or an enum value/tag; `#[py(skip)]` omits a field or variant.

Generated models describe types and can be instantiated in Python; they do not implement JSON serialization or convert Rust values. The current derive supports owned, non-generic structs and enums. Imports are emitted after declarations so cyclic references remain importable and `typing.get_type_hints()` can resolve them.
