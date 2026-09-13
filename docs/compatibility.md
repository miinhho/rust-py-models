# Python compatibility and checks

Generated modules are intended for CPython 3.10 through 3.14. The [CI workflow](../.github/workflows/ci.yml) is configured to run Rust formatting, Clippy, and tests once, upload the generated `py-rs/bindings` package, then check that same package independently under each of Python 3.10, 3.11, 3.12, 3.13, and 3.14.

Each Python job compiles all generated modules, runs Ruff on the generated package and runtime tests, and executes the runtime tests. Those tests import real generated classes, construct dataclasses and enum variants, check `frozen`/`slots`/`kw_only`, and resolve cyclic and generic annotations with `typing.get_type_hints()`. Ruff also checks unused imports, undefined names, import order, and late-import suppressions.

Run the checks locally after generating bindings:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
cargo test --workspace --all-features --locked
python -m compileall -q py-rs/bindings
ruff check --select E402,F401,F821,I py-rs/bindings py-rs/tests/test_generated_bindings.py
python -m unittest discover -s py-rs/tests -p 'test_*.py'
```

Run the final three commands with each Python version you want to verify. A local pass on one version does not establish compatibility with the other versions; the CI matrix reports those results separately.
