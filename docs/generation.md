# Generation and export

`#[derive(PY)]` analyzes a Rust struct or enum and implements the typed [`PY`](../rust-py-models/src/trait_py.rs) contract. The derive builds `TypeSpec` and `ModelSpec` IR; a single renderer turns that IR into Python source when an export method runs. Compilation does not write Python files. `#[py(export)]` also creates a Rust test that calls `export_all()`; running `cargo test export_bindings` executes that test and writes the files.

`export_all()` checks the root and its transitive dependencies before writing generated modules. Declarations with the same `#[py(export_to)]` path are combined into one module with deduplicated imports and definitions. `export_all_to(path)` performs the same operation in an explicit base directory and ignores `RUST_PY_MODELS_EXPORT_DIR`. `export()` writes only the selected declaration, while `export_to_string()` renders its module without writing it. The default output directory is `bindings/`, relative to the Rust package running the test. Set `RUST_PY_MODELS_EXPORT_DIR`, for example `RUST_PY_MODELS_EXPORT_DIR=python/bindings cargo test export_bindings`, to change it. `#[py(export_to = "models/User.py")]` sets a path within that directory. Nested output directories receive `__init__.py` package markers. A locked owner manifest records each export root's generated files, removes obsolete files only when no other root owns them, and refuses to replace or delete generated files modified since the previous export. Output paths that differ only by ASCII case are rejected for portability.

Generated modules begin with a provenance header, `# fmt: off`, and `from __future__ import annotations`. Their standard-library imports and non-cyclic model imports precede the declarations. Imports are aliased when a Python field would shadow a module or imported model name. Imports that would form a dependency cycle follow the declarations with `# noqa: E402 - cyclic dependency`. This allows the modules to import each other after their classes have been defined; `typing.get_type_hints()` can then resolve the annotations.

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

`EventLogin(user_id=1)` and `EventLogout(user_id=1)` are distinct runtime objects even though their fields match. `Event` is a type alias, not a base class or an object to construct. Without Serde tagging, no synthetic tag field is added. Unit-only enum values default to their Rust variant names as strings; Rust discriminant values do not set the Python values.

## Attributes

| Attribute | Applies to | Result |
| --- | --- | --- |
| `#[py(export)]` | Struct or enum | Adds an export test that includes transitive dependencies. |
| `#[py(export_to = "models/User.py")]` | Struct or enum | Sets the path below the export directory. |
| `#[py(rename = "...")]` | Struct, enum, field, or variant | Renames the Python class or field. On a unit-only enum variant, changes its string value; on a payload variant, changes the suffix of its generated class name. |
| `#[py(skip)]` | Field or variant | Omits it from the Python declaration. |
| `#[py(unsafe_type = "decimal.Decimal", import = "decimal")]` | Field | Injects an unchecked annotation and optionally adds a module import. The name is deliberately unsafe: the generator does not parse the annotation, rewrite names inside it, generate conversions, or verify that it matches the Rust type. |
| `#[py(as = "RustType")]` | Struct, enum, or field | Uses another Rust type's Python mapping. On a struct or enum this creates no Python declaration or output file; on a field it replaces that field's inferred mapping. |
| `#[py(newtype)]` | One-field non-generic tuple struct | Emits a `typing.NewType` declaration that preserves static type identity without wrapping the runtime Python value. |
| `#[py(concrete(T = RustType))]` | Generic struct or enum | Substitutes a Rust type for a generic parameter in the generated declaration and removes that parameter from Python `Generic[...]`. |
| `#[py(bound = "T: rust_py_models::PY")]` | Generic struct or enum | Supplies explicit Rust where-predicates instead of inferred `PY` bounds. |
| `#[py(frozen)]` | Struct, payload enum, or payload variant | Uses `@dataclass(frozen=True)`. |
| `#[py(slots)]` | Struct, payload enum, or payload variant | Uses `@dataclass(slots=True)`. |
| `#[py(kw_only)]` | Struct, payload enum, or payload variant | Uses `@dataclass(kw_only=True)`. |

With the optional `serde-compat` Cargo feature, the derive recognizes Serde `rename`, `rename_all`, `rename_all_fields`, `tag`, `content`, `untagged`, `skip`, `flatten`, and `default` attributes. The supported rename rules are Serde's `lowercase`, `UPPERCASE`, `PascalCase`, `camelCase`, `snake_case`, `SCREAMING_SNAKE_CASE`, `kebab-case`, and `SCREAMING-KEBAB-CASE`. `#[py(rename = "...")]` takes precedence over Serde rename on the same item.

`tag` adds an `init=False` `Literal[...]` discriminator to structs and to unit or named enum variants. `tag` plus `content` uses an adjacent discriminator and payload field; named payloads receive a generated `...Content` dataclass. `untagged` payload enums retain their variant dataclasses without a discriminator. `rename_all_fields` applies to named payload fields. `flatten` expands fields from another derived struct and validates duplicate Python names. `skip` omits a field or variant. `default` is accepted for source compatibility but does not add a Python default because the Rust default value is not available to the derive; generated constructor arguments remain required.

Dataclass flags default to Python's `False`. Explicit values such as `#[py(slots = false)]` are accepted. Flags on a payload enum apply to all generated variant classes, and each variant can override them. Unit-only enums reject dataclass flags. Python names and output paths are checked for invalid ASCII identifiers and collisions; invalid Serde renames are rejected rather than rewritten.

Rust doc comments become Python class or attribute docstrings. `#[deprecated(note = "...", since = "...")]` is appended to the generated documentation. Rust examples are preserved only as ordinary documentation text; the generator does not translate Rust examples into Python.

Type parameters become Python `TypeVar` and `Generic` declarations. Rust lifetimes disappear from Python annotations. `PhantomData<T>` fields are omitted. `#[py(concrete(...))]` specializes selected parameters, while `#[py(bound = "...")]` replaces inferred `PY` bounds. Const generics and generic `#[py(newtype)]` declarations are unsupported. Because `#[py(export)]` cannot choose concrete Rust type arguments for a generic declaration, export a concrete instantiation from a handwritten Rust test, for example `Page::<User>::export_all()`.

See [type mapping](type-mapping.md) for the annotations used inside these classes and [compatibility](compatibility.md) for runtime checks.
