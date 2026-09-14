# Generate and export Python models

## Export a model graph

Derive `PY` for every Rust struct or enum that should appear in Python. Mark each export root and call `rust_py_models::export_all()` from a Rust binary or other explicit entry point:

```rust
use rust_py_models::PY;

#[derive(PY)]
struct Address {
    city: String,
}

#[derive(PY)]
#[py(export, export_to = "models/User.py")]
struct User {
    id: u64,
    address: Address,
}

fn main() -> Result<(), rust_py_models::ExportError> {
    rust_py_models::export_all()?;
    Ok(())
}
```

Put the example in `src/main.rs` and run `cargo run` to write `models/User.py`, `Address.py`, and package `__init__.py` files below `bindings/` in the current working directory. Each referenced model keeps its own `export_to` path; without one, `Address` uses `Address.py`. `#[py(export)]` registers the root in the linked program; files are written only when an export API is called.

Choose the scope and output location:

| API | Use when |
| --- | --- |
| `rust_py_models::export_all()` | Export every registered root and its dependencies to `bindings/` or the configured directory. |
| `rust_py_models::export_all_to(path)` | Export every registered root to the specified directory. |
| `User::export_all()` | Export one root and its dependencies to the configured directory. |
| `User::export_all_to(path)` | Export one root and its dependencies to the specified directory. |
| `User::export()` | Write only `User`'s declaration. Referenced models are not exported. |
| `User::export_to_string()` | Get rendered Python as a string without writing a file. |

`export_all_to(path)` ignores `RUST_PY_MODELS_EXPORT_DIR`. Paths from `#[py(export_to = "...")]` remain relative to the supplied directory.

`rust_py_models::export_all()` and `rust_py_models::export_all_to(path)` return an error if the linked program contains no `#[py(export)]` roots.

## Generated file ownership

Use a dedicated directory for generated files and keep its ownership manifest. On regeneration, the exporter:

- updates generated files whose previous contents have not been modified;
- rejects handwritten or modified files at a target path;
- removes a dependency's generated file when an exported root stops referencing it and no other root owns it;
- rejects output paths that differ only by ASCII case.

Removing an entire `#[py(export)]` root does not remove files it generated earlier. Delete those files yourself. To change a generated module, edit the Rust model or its `#[py(...)]` options and regenerate.

Declarations that share an `export_to` path are emitted in one module. Imports and shared definitions are deduplicated. A field or declaration that shadows an imported name causes the import to receive a deterministic alias.

## Generated class shapes

Named structs become dataclasses with named fields. Tuple structs use `_0`, `_1`, and subsequent positional field names. Unit structs become empty dataclasses.

A unit-only enum becomes a `str, Enum` class. An enum with payload variants becomes one dataclass per variant and a union alias:

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

Construct `EventLogin` or `EventLogout`; `Event` is an annotation, not a base class. Unit-only enum values use the Rust variant name unless renamed. Rust discriminants do not become Python values.

## Configure generated declarations

| Attribute | Applies to | Effect |
| --- | --- | --- |
| `#[py(export)]` | Struct or enum | Registers a root for `rust_py_models::export_all()` and `export_all_to(path)`. |
| `#[py(export_to = "models/User.py")]` | Struct or enum | Sets the module path below the export directory. |
| `#[py(rename = "...")]` | Struct, enum, field, or variant | Changes the Python name. For unit enum variants it changes the string value; for payload variants it changes the generated class suffix. |
| `#[py(skip)]` | Field or variant | Omits the item from Python. |
| `#[py(as = "RustType")]` | Struct, enum, or field | Reuses another Rust type's Python mapping. |
| `#[py(unsafe_type = "decimal.Decimal", import = "decimal")]` | Field | Emits an unchecked annotation and optional module import. |
| `#[py(newtype)]` | One-field, non-generic tuple struct | Emits a `typing.NewType` declaration for static type identity. |
| `#[py(concrete(T = RustType))]` | Generic struct or enum | Replaces a type parameter with a concrete Rust mapping. |
| `#[py(bound = "T: rust_py_models::PY")]` | Generic struct or enum | Replaces inferred `PY` bounds with the supplied predicates. |
| `#[py(frozen)]` | Struct or payload variant | Sets `frozen=True` on the dataclass. |
| `#[py(slots)]` | Struct or payload variant | Sets `slots=True` on the dataclass. |
| `#[py(kw_only)]` | Struct or payload variant | Sets `kw_only=True` on the dataclass. |

`unsafe_type` is unchecked. The optional `import` adds a Python module import, but the exporter does not install the module, verify the annotation, or export models mentioned inside it. Prefer `as` when an existing Rust type has the mapping you need.

`Result<T, E>` fields use the mapping for `T` by default, including results inside collections. Use `as` on a field when its Python representation differs. The exporter only generates annotations; it does not turn `Err` into an exception or remove error values.

Dataclass options default to `False`; explicit `false` values are accepted. Options on a payload enum apply to every generated variant unless a variant overrides them. Unit-only enums reject dataclass options.

## Use Serde names and shapes

Enable the `serde-compat` feature to read these Serde attributes:

- `rename`, `rename_all`, and `rename_all_fields`;
- `tag`, `content`, and `untagged`;
- `skip`, `flatten`, and `default`.

`#[py(rename = "...")]` takes precedence over a Serde rename on the same item. Invalid Python identifiers and duplicate Python names are errors rather than silently rewritten.

Serde `tag` adds an `init=False` `Literal[...]` discriminator. `tag` with `content` emits an adjacent payload field and, for named payloads, a content dataclass. `flatten` copies fields from another derived dataclass and rejects duplicate names. Serde `default` does not make a Python constructor argument optional.

## Generics and documentation

Rust type parameters become Python `TypeVar` and `Generic` declarations. Lifetimes are omitted, and `PhantomData<T>` fields are not generated. Const generics and generic `#[py(newtype)]` declarations are unsupported.

`#[py(export)]` cannot be placed on a generic root. Export a concrete instantiation from your Rust entry point:

```rust
Page::<User>::export_all()?;
```

Rust doc comments become Python class or attribute documentation. A `#[deprecated(note = "...", since = "...")]` annotation is included in that documentation.

See [type mapping](type-mapping.md) for field annotations. Supported Rust and Python versions are listed in the [README](../README.md#requirements).
