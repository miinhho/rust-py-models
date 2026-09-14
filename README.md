# rust-py-models

`rust-py-models` generates importable Python dataclasses, enums, and type aliases from Rust types. Use it when Rust owns the model definitions and Python consumers need matching classes without maintaining a second schema.

The generated files are ordinary Python modules. They are not `.pyi` stubs, an FFI layer, or a serializer. Python callers can instantiate the generated classes, but annotations are not runtime validation.

## Requirements

The crates require Rust 1.98 or newer. Generated modules target CPython 3.10 through 3.14.

## Quick start

Add the crate to the Rust package that owns the models:

```toml
[dependencies]
rust-py-models = "0.1"
```

Derive `PY` for referenced types and add `#[py(export)]` to each export root:

```rust
use rust_py_models::PY;

#[derive(PY)]
struct Address {
    city: String,
}

#[derive(PY)]
#[py(export)]
struct User {
    id: u64,
    address: Option<Address>,
}
```

Generate the modules by running the export tests:

```sh
cargo test export_bindings
```

The default output is `bindings/` in the package directory:

```text
bindings/
├── __init__.py
├── Address.py
└── User.py
```

Python can import and construct the generated classes:

```python
from bindings.Address import Address
from bindings.User import User

user = User(id=1, address=Address(city="Seoul"))
```

`Option<Address>` allows `None`, but it does not make the constructor argument optional. The generated class does not validate annotations or convert Rust values.

## Choose an export workflow

- Use `#[py(export)]` when generation should run through a Rust test.
- Use `PY::export_all()` to generate a root and all referenced models from Rust code.
- Use `PY::export_all_to(path)` when the caller chooses the output directory.
- Use `PY::export_to_string()` when another tool owns file output.

Set `RUST_PY_MODELS_EXPORT_DIR` to change the output directory used by generated export tests and `export_all()`.

Generated files carry an ownership header. The exporter updates files it owns, removes obsolete owned files, and refuses to overwrite user-modified or unrelated files.

## Guides

- [Generate models and configure `#[py(...)]`](docs/generation.md)
- [Find the Python annotation for a Rust type](docs/type-mapping.md)
- [Check supported Python versions](docs/compatibility.md)
