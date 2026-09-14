# Generate Python models

## Export a model graph

Derive `PY` for every Rust struct or enum that should appear in Python. Mark an export root with `#[py(export)]`:

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
```

Run `cargo test export_bindings` to write `models/User.py`, `Address.py`, and package `__init__.py` files below the export directory. Each referenced model keeps its own `export_to` path; without one, `Address` uses `Address.py`. The default export directory is `bindings/`. Set `RUST_PY_MODELS_EXPORT_DIR` to choose another location.

Choose the API according to who owns generation:

| API | Use when |
| --- | --- |
| `#[py(export)]` | A Rust test should regenerate the model and all dependencies. |
| `PY::export_all()` | Rust code should export a model graph to the configured directory. |
| `PY::export_all_to(path)` | Rust code should choose the base output directory explicitly. |
| `PY::export()` | Only the selected declaration should be written. |
| `PY::export_to_string()` | The caller needs rendered Python without file output. |

`export_all_to(path)` ignores `RUST_PY_MODELS_EXPORT_DIR`. Paths from `#[py(export_to = "...")]` remain relative to the supplied directory.

## Generated file ownership

Generated modules contain a provenance header. The exporter uses that ownership information to avoid destroying handwritten code:

- a generated file may be replaced by the Rust type that owns it;
- a modified generated file is rejected instead of overwritten;
- an obsolete generated file is removed only after no export root owns it;
- output paths that differ only by ASCII case are rejected for portability.

Do not hand-edit generated modules. Change the Rust model or generator configuration and regenerate them.

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
| `#[py(export)]` | Struct or enum | Adds an export test for the model graph. |
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

`unsafe_type` is an escape hatch. The exporter does not parse the expression, rewrite names inside it, export referenced models, or verify that it represents the Rust type. Prefer `as` when an existing Rust type already has the required mapping.

Dataclass options default to `False`; explicit `false` values are accepted. Options on a payload enum apply to every generated variant unless a variant overrides them. Unit-only enums reject dataclass options.

## Use Serde names and shapes

Enable the `serde-compat` feature to read these Serde attributes:

- `rename`, `rename_all`, and `rename_all_fields`;
- `tag`, `content`, and `untagged`;
- `skip`, `flatten`, and `default`.

`#[py(rename = "...")]` takes precedence over a Serde rename on the same item. Invalid Python identifiers and duplicate Python names are errors rather than silently rewritten.

Serde `tag` adds an `init=False` `Literal[...]` discriminator. `tag` with `content` emits an adjacent payload field and, for named payloads, a content dataclass. `flatten` copies fields from another derived dataclass and rejects duplicate names. Serde `default` does not make a Python argument optional because the derive cannot obtain the Rust default value.

## Generics and documentation

Rust type parameters become Python `TypeVar` and `Generic` declarations. Lifetimes are omitted, and `PhantomData<T>` fields are not generated. Const generics and generic `#[py(newtype)]` declarations are unsupported.

An export test cannot infer concrete arguments for a generic root. Export a concrete instantiation from a handwritten Rust test:

```rust
Page::<User>::export_all()?;
```

Rust doc comments become Python class or attribute documentation. A `#[deprecated(note = "...", since = "...")]` annotation is included in that documentation.

See [type mapping](type-mapping.md) for field annotations and [compatibility](compatibility.md) for supported Python versions.
