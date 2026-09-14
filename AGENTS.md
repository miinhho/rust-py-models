# Repository instructions

## Purpose

`rust-py-models` generates importable Python models from Rust structs and enums. Rust definitions are the source of truth. Generated Python is a consumer artifact, not handwritten source, an FFI layer, or a runtime validator.

## Repository layout

- `rust-py-models/` — public `PY` trait, IR, type mappings, renderer, and file export.
- `rust-py-models-macros/` — `#[derive(PY)]` and `#[py(...)]` parsing and expansion.
- `rust-py-models/tests/python/` — Rust fixtures and generation contracts.
- `python-tests/runtime/` — behavior visible to Python callers.
- `python-tests/typecheck/` — strict mypy contracts.
- `python-tests/binding/` — generated files; never edit them directly.
- `docs/` — user guides and reference documentation.

Use sibling module roots (`src/export.rs` with `src/export/`), not `mod.rs`. Put each optional external crate mapping in `rust-py-models/src/impls/external/<crate>.rs` and gate it in `external.rs`.

## Toolchain

Use the Rust version pinned by `rust-toolchain.toml`; do not substitute a floating `stable` toolchain. Keep the pinned channel, both crates' `rust-version`, and the CI toolchain version aligned.

## Design constraints

- Keep Rust-to-Python mappings explicit and deterministic.
- Prefer structured `TypeSpec` constructors over `TypeSpec::unsafe_raw`.
- Treat `unsafe_raw` as an unchecked escape hatch: it does not add imports, dependencies, conversion, or validation.
- Preserve importability for nested modules, name collisions, and cyclic model graphs.
- Keep output paths portable across case-sensitive and case-insensitive filesystems.
- Never overwrite handwritten or user-modified Python files.
- Do not add aliases, deprecated paths, or compatibility shims during a clean cutover unless requested.

## Changing generated behavior

Update all three layers when their contract changes:

1. derive expansion or Rust type mapping;
2. renderer/export behavior;
3. a Python runtime or typecheck consumer when Python-visible behavior changes.

Use Rust tests for validation and generation errors. Use Python tests for behavior a Python caller or type checker can observe. Do not assert source formatting when a semantic assertion is available.

## Documentation and comments

Write for the reader performing a task:

- lead with what the API guarantees or when to use it;
- document errors, side effects, ownership, and unsupported behavior;
- keep implementation history, debugging notes, and design scratch work out of user docs;
- use Rustdoc for public contracts and ordinary comments only for non-obvious invariants or reasons;
- do not narrate code that is already clear from names and types;
- update mapping and generation docs in the same change as user-visible behavior.

## Releases

Publish only through `.github/workflows/release.yml` and the protected `crates-io` environment. Keep the macro crate dependency version aligned and publish the macro crate before `rust-py-models`.

## Verification

Run formatting and Clippy for every Rust change:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

Run the complete generation and Python consumer pipeline for any renderer, derive, mapping, export, fixture, or generated-output change:

```sh
uv run --project python-tests --locked python python-tests
```

The pipeline regenerates `python-tests/binding/`, checks Python formatting and imports, runs strict mypy, imports each module independently, and runs the runtime suite.

For documentation-only changes, build Rustdoc and check links:

```sh
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked
```
