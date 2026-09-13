# py-rs

Generate Python 3.10+ type models from Rust structs and enums. Like [ts-rs](https://github.com/Aleph-Alpha/ts-rs), `py-rs` uses a derive macro to inspect Rust types at compile time, a trait to describe the target type, and an export step to write bindings during `cargo test`.

## Quick start

Add the crate to a Rust project:

```toml
[dependencies]
py-rs = { path = "path/to/py-rs/py-rs" }
```

Derive `PY` and mark a root type for export:

```rust
use py_rs::PY;

#[derive(PY)]
#[py(export_to = "models/Address.py", frozen, slots)]
struct Address {
    city: String,
}

#[derive(PY)]
#[py(export, frozen, slots, kw_only)]
struct User {
    id: u64,
    address: Option<Address>,
}
```

Run `cargo test export_bindings`. The generated test exports `User` and its dependency `Address` to `bindings/User.py` and `bindings/models/Address.py` in the Rust package running the test. `User.py` contains:

```python
from __future__ import annotations

from dataclasses import dataclass

from .models.Address import Address


@dataclass(frozen=True, slots=True, kw_only=True)
class User:
    id: int
    address: Address | None
```

Set `PY_RS_EXPORT_DIR` to change the base output directory:

```sh
PY_RS_EXPORT_DIR=python/bindings cargo test export_bindings
```

You can also call `User::export()` to write only `User`, `User::export_all()` to include transitive dependencies, or `User::export_to_string()` to inspect its generated module without writing a file. The `PY` trait also exposes `name()`, `inline()`, `decl()`, and `dependencies()` for programmatic use.

## Attributes

| Attribute | Applies to | Effect |
| --- | --- | --- |
| `#[py(export)]` | Struct or enum | Add a `cargo test` export test. |
| `#[py(export_to = "models/User.py")]` | Struct or enum | Set the path relative to the export directory. |
| `#[py(rename = "...")]` | Struct, enum, field, or variant | Rename the Python class or field, or the enum value/tag. |
| `#[py(skip)]` | Field or variant | Omit it from the Python model. |
| `#[py(frozen)]` | Struct, payload enum, or its variant | Generate `@dataclass(frozen=True)`. |
| `#[py(slots)]` | Struct, payload enum, or its variant | Generate `@dataclass(slots=True)`. |
| `#[py(kw_only)]` | Struct, payload enum, or its variant | Generate `@dataclass(kw_only=True)`. |

Dataclass options default to Python's `False`. The flag form sets `True`; an explicit value also works, such as `#[py(slots = false)]`. On an enum with payloads, options on the enum apply to every variant class. A variant can override them:

```rust
#[derive(PY)]
#[py(frozen, slots)]
enum Event {
    Created { id: u64 },
    #[py(frozen = false, slots = false)]
    Mutable(String),
}
```

Unit-only enums become Python `str, Enum` classes rather than dataclasses, so dataclass options are rejected on them.

## Generated models

Named, tuple, and unit structs become dataclasses. Unit-only enums become `str, Enum` classes. Enums with payloads become a dataclass for each variant plus a `TypeAlias` union; each variant has a `kind` tag. Direct imports appear at the top of a module. Imports that would form a cycle remain after the declarations with an explicit `E402` suppression, allowing Python import and `typing.get_type_hints()` to resolve the cycle.

| Rust | Python annotation |
| --- | --- |
| Integer types | `int` |
| Float types | `float` |
| `bool` | `bool` |
| `String`, `char` | `str` |
| `Option<T>` | `T | None` |
| `Vec<T>`, maps, sets | `list[T]`, `dict[K, V]`, `set[T]` |
| `Box<T>` | `T` |
| Arrays and 2-tuples | `tuple[...]` |

The generated files are importable Python models, not Rust FFI bindings. They do not serialize JSON or convert Rust values. The current derive supports owned, non-generic structs and enums.
