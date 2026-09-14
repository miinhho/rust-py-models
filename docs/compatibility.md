# Python compatibility and checks

Generated modules are intended for CPython 3.10 through 3.14. The [CI workflow](../.github/workflows/ci.yml) is configured to run Rust formatting, Clippy, and tests once, upload the generated `python-tests/binding` package, then check that same package independently under each of Python 3.10, 3.11, 3.12, 3.13, and 3.14.

Rust fixtures whose generated bindings are consumed by Python live under
`rust-py-models/tests/python/`. The independent Python consumer runner lives at
`python-tests/`; its runtime and static contracts are split by behavior under
`runtime/` and `typecheck/`. Generated files shared between the Rust producer
and Python consumer are written to `python-tests/binding/`. Each Python job
compiles every generated module, runs Ruff formatting checks (generated files opt out with `# fmt: off`) and lint rules, checks generated modules and consumer examples with mypy, imports every generated module in a fresh interpreter, and executes runtime tests. Runtime coverage includes dataclass constructor semantics, documentation, enum representations, nested cyclic imports, recursive generics, temporal type identity, standard mappings, and optional Cargo feature mappings.

The Python test project owns its `python-tests/pyproject.toml`,
`python-tests/uv.lock`, `python-tests/.python-version`, and local `.venv`.
Run the complete Rust-generation-to-Python-consumer pipeline locally:

```sh
uv sync --project python-tests --locked
uv run --project python-tests --locked python python-tests
```

The runner removes and regenerates `python-tests/binding/` before checking it,
so files from an earlier run cannot mask a missing export. To check bindings
already generated in that directory, run
`uv run --project python-tests --locked python python-tests --check-only`.
The CI matrix overrides the local default interpreter with each supported
Python version while installing the exact dependency graph recorded in
`python-tests/uv.lock`. A local pass on one version does not establish compatibility with
the other versions; the CI matrix reports those results separately.
