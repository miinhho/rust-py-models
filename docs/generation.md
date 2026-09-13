# Generation and export

`#[derive(PY)]` analyzes a Rust struct or enum at compile time and implements the `PY` trait. The trait supplies its Python name, declaration, imports, referenced types, and output path. Compilation does not write Python files. `#[py(export)]` also creates a Rust test that calls `export_all()`; running `cargo test export_bindings` executes that test and writes the files.

`export_all()` checks the root and its transitive dependencies before writing one module per derived declaration. `export()` writes only the selected declaration, while `export_to_string()` renders its module without writing it. The default output directory is `bindings/`, relative to the Rust package running the test. Set `PY_RS_EXPORT_DIR`, for example `PY_RS_EXPORT_DIR=python/bindings cargo test export_bindings`, to change it. `#[py(export_to = "models/User.py")]` sets a path within that directory. Nested output directories receive `__init__.py` files. Existing hand-written files and files generated for a different Rust declaration are not overwritten.

Generated modules begin with a provenance header and `from __future__ import annotations`. Their standard-library imports and non-cyclic model imports precede the declarations. Imports that would form a dependency cycle follow the declarations with `# noqa: E402 - cyclic dependency`. This allows the modules to import each other after their classes have been defined; `typing.get_type_hints()` can then resolve the annotations.

## Python class shapes

Named structs become dataclasses with named fields. Tuple structs become dataclasses with `_0`, `_1`, and so on; unit structs become empty dataclasses. A unit-only enum becomes a `str, Enum` class. An enum containing payload variants generates one dataclass per variant, including empty dataclasses for its unit variants, and a `TypeAlias` union for the enum name:

```rust
#[derive(PY)]
enum Event {
    Login { user_id: i32 },
    Logout { user_id: i32 },
}
```

```python
@dataclass
class EventLogin:
    user_id: int

@dataclass
class EventLogout:
    user_id: int

Event: TypeAlias = EventLogin | EventLogout
```

`EventLogin(user_id=1)` and `EventLogout(user_id=1)` are distinct runtime objects even though their fields match. `Event` is a type alias, not a base class or an object to construct. No synthetic tag field is added. Unit-only enum values default to their Rust variant names as strings; Rust discriminant values and Serde attributes do not set the Python values.

## Attributes

| Attribute | Applies to | Result |
| --- | --- | --- |
| `#[py(export)]` | Struct or enum | Adds an export test that includes transitive dependencies. |
| `#[py(export_to = "models/User.py")]` | Struct or enum | Sets the path below the export directory. |
| `#[py(rename = "...")]` | Struct, enum, field, or variant | Renames the Python class or field. On a unit-only enum variant, changes its string value; on a payload variant, changes the suffix of its generated class name. |
| `#[py(skip)]` | Field or variant | Omits it from the Python declaration. |
| `#[py(type = "decimal.Decimal", import = "decimal")]` | Field | Overrides its annotation and optionally adds a module import. This does not generate a conversion or validate the override against the Rust type. |
| `#[py(frozen)]` | Struct, payload enum, or payload variant | Uses `@dataclass(frozen=True)`. |
| `#[py(slots)]` | Struct, payload enum, or payload variant | Uses `@dataclass(slots=True)`. |
| `#[py(kw_only)]` | Struct, payload enum, or payload variant | Uses `@dataclass(kw_only=True)`. |

Dataclass flags default to Python's `False`. Explicit values such as `#[py(slots = false)]` are accepted. Flags on a payload enum apply to all generated variant classes, and each variant can override them. Unit-only enums reject dataclass flags. Python names and output paths are checked for invalid identifiers and collisions.

Type parameters become Python `TypeVar` and `Generic` declarations. Rust lifetimes disappear from Python annotations. `PhantomData<T>` fields are omitted. Const generics are unsupported. Because `#[py(export)]` cannot choose concrete type arguments for a generic declaration, export a concrete instantiation from a handwritten Rust test, for example `Page::<User>::export_all()`.

See [type mapping](type-mapping.md) for the annotations used inside these classes and [compatibility](compatibility.md) for runtime checks.
