# Python compatibility and checks

Generated modules are intended for CPython 3.10 through 3.14. The [CI workflow](../.github/workflows/ci.yml) is configured to run Rust formatting, Clippy, and tests once, upload the generated `py-rs/bindings` package, then check that same package independently under each of Python 3.10, 3.11, 3.12, 3.13, and 3.14.

Each Python job runs the same Python test command used locally. It compiles generated modules, runs Ruff, checks generated modules and consumer examples with mypy, and executes runtime tests. The consumer examples include valid calls and invalid calls marked with specific `type: ignore[...]` codes; mypy's unused-ignore check fails if an invalid call is no longer detected. Runtime tests construct real classes and resolve cyclic and generic annotations with `typing.get_type_hints()`.

Create a Python test environment and run the complete generation-to-consumer pipeline locally:

```sh
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements-test.txt
.venv/bin/python scripts/test_python.py
```

The command exports bindings into a fresh temporary directory before checking them, so ignored files from an earlier run cannot mask a missing export. To check bindings generated elsewhere, run `.venv/bin/python scripts/test_python.py --bindings-dir path/to/bindings`. The CI Rust job still runs formatting, Clippy, and Rust tests, then passes its generated package to each Python matrix job. A local pass on one version does not establish compatibility with the other versions; the CI matrix reports those results separately.
