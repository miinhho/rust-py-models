# rust-py-models

`rust-py-models` generates importable Python dataclasses, enums, and type aliases from Rust types. Use it when Rust owns the model definitions and Python consumers need matching classes without maintaining a second schema.

The generated files are ordinary Python modules. They are not `.pyi` stubs, an FFI layer, or a serializer. Python callers can instantiate the generated classes, but annotations are not runtime validation.

## Requirements

The crates require Rust 1.98 or newer. Generated modules target CPython 3.10 through 3.14 and can be imported without a Rust runtime in the Python environment.

## Quick start

Add the crate to the Rust package that owns the models:

```toml
[dependencies]
rust-py-models = "0.1.1"
```

Derive `PY` for referenced types, mark the models you want to export as roots, and call an export API from a Rust entry point:

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

fn main() -> Result<(), rust_py_models::ExportError> {
    rust_py_models::export_all()?;
    Ok(())
}
```

Run that entry point to generate the modules. For example, if it is in `src/main.rs`:

```sh
cargo run
```

This writes `bindings/` relative to the current working directory:

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

- Use `rust_py_models::export_all()` to generate all `#[py(export)]` roots linked into the program and their referenced models.
- Use `rust_py_models::export_all_to(path)` to choose the directory for all registered roots.
- Use `User::export_all()` or `User::export_all_to(path)` to export one selected root and its referenced models.
- Use `PY::export()` to write only the selected declaration.
- Use `PY::export_to_string()` when another tool owns file output.

Set `RUST_PY_MODELS_EXPORT_DIR` to change the output directory used by APIs without an explicit path. `#[py(export)]` only registers a root; files are written when an export API is called.

Generated files carry an ownership header. The exporter updates files it owns, removes obsolete owned files, and refuses to overwrite user-modified or unrelated files.

## Guides

- [Generate models and configure `#[py(...)]`](docs/generation.md)
- [Find the Python annotation for a Rust type](docs/type-mapping.md)
