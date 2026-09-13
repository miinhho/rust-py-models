# py-rs

Generate importable Python classes from Rust structs and enums. Rust is the source of truth: Python users construct generated dataclasses and enums instead of maintaining a second set of model definitions. The output is Python `.py` code, not a `.pyi` stub or a Rust FFI layer.

Like [ts-rs](https://github.com/Aleph-Alpha/ts-rs), `py-rs` uses a derive macro at compile time and exports bindings when a Rust test runs.

## Quick start

Add `py-rs` to a Rust project:

```toml
[dependencies]
py-rs = { path = "path/to/py-rs/py-rs" }
```

```rust
use py_rs::PY;

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

Run `cargo test export_bindings` in that Rust package. This writes `bindings/User.py` and its dependency `bindings/Address.py`. Python can then construct real objects:

```python
from bindings.Address import Address
from bindings.User import User

user = User(id=1, address=Address(city="Seoul"))
```

`Option<Address>` permits `None`, but the `address` constructor argument is still required. Generated annotations do not validate values at runtime or convert Rust values.

## Documentation

- [Generation, export, enums, and `#[py(...)]` options](docs/generation.md)
- [Supported Rust types and Python mappings](docs/type-mapping.md)
- [Python compatibility and development checks](docs/compatibility.md)
