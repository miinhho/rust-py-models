# Compatibility

## Rust

The crates require Rust 1.98 or newer. `rust-toolchain.toml` pins Rust 1.98.0, including Rustfmt and Clippy, so local commands and CI use the same toolchain.

## Python

Generated modules target CPython 3.10 through 3.14. They use annotations and standard-library dataclass features available throughout that range.

Compatibility means that generated modules:

- compile as Python source;
- can be imported independently;
- pass Ruff's syntax and import checks;
- pass strict mypy checks together with the consumer examples;
- preserve the documented dataclass, enum, generic, and import behavior.

It does not mean that annotations perform runtime validation, that Python objects serialize exactly like Serde values, or that generated classes convert values to and from Rust.

## Validate a change locally

Install the locked Python tools and run the complete producer-to-consumer check:

```sh
uv sync --project python-tests --locked
uv run --project python-tests --locked python python-tests
```

The runner removes `python-tests/binding/`, runs the Rust tests to regenerate it, and then checks the new package. This prevents stale files from hiding a missing export.

To validate bindings that already exist in `python-tests/binding/` without regenerating them:

```sh
uv run --project python-tests --locked python python-tests --check-only
```

The complete local command uses the active Python interpreter. CI repeats the Python checks on 3.10, 3.11, 3.12, 3.13, and 3.14.

## Test layout

- `rust-py-models/tests/python/` contains Rust fixtures and generation assertions.
- `python-tests/runtime/` checks behavior visible to Python callers.
- `python-tests/typecheck/` checks static typing contracts.
- `python-tests/binding/` is generated output and must not be edited by hand.

A change to generated Python behavior should update the Rust fixture that produces it and the Python consumer contract that observes it.
