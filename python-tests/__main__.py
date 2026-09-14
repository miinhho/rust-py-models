"""Generate and validate Python bindings with the active interpreter."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TEST_ROOT = Path(__file__).resolve().parent
BINDINGS = TEST_ROOT / "binding"
RUNTIME_TESTS = TEST_ROOT / "runtime"
TYPECHECK_TESTS = TEST_ROOT / "typecheck"


def run(*args: str, env: dict[str, str] | None = None) -> None:
    subprocess.run(args, cwd=ROOT, env=env, check=True)


def environment(bindings: Path) -> dict[str, str]:
    env = os.environ.copy()
    root = str(bindings.parent)
    existing_python_path = env.get("PYTHONPATH")
    env["PYTHONPATH"] = (
        root
        if existing_python_path is None
        else root + os.pathsep + existing_python_path
    )
    env["MYPYPATH"] = root
    return env


def binding_modules(bindings: Path) -> list[str]:
    modules = []
    for source in sorted(bindings.rglob("*.py")):
        if source.name == "__init__.py":
            continue
        relative = source.relative_to(bindings).with_suffix("")
        modules.append(".".join((bindings.name, *relative.parts)))
    return modules


def check_isolated_imports(bindings: Path, env: dict[str, str]) -> None:
    program = "import importlib, sys; importlib.import_module(sys.argv[1])"
    for module in binding_modules(bindings):
        run(sys.executable, "-c", program, module, env=env)


def check(bindings: Path) -> None:
    if not (bindings / "__init__.py").is_file() or not (bindings / "User.py").is_file():
        raise SystemExit(f"Bindings are missing from {bindings}")

    version = f"{sys.version_info.major}.{sys.version_info.minor}"
    targets = (
        str(bindings),
        str(RUNTIME_TESTS),
        str(TYPECHECK_TESTS),
        str(Path(__file__)),
    )
    with tempfile.TemporaryDirectory(prefix="rust-py-models-pycache-") as pycache:
        env = environment(bindings)
        env["PYTHONPYCACHEPREFIX"] = pycache

        run(sys.executable, "-m", "compileall", "-q", *targets, env=env)
        run(sys.executable, "-m", "ruff", "format", "--check", *targets, env=env)
        run(
            sys.executable,
            "-m",
            "ruff",
            "check",
            "--select",
            "E4,E7,E9,F,I",
            *targets,
            env=env,
        )
        run(
            sys.executable,
            "-m",
            "mypy",
            "--strict",
            "--no-incremental",
            "--python-version",
            version,
            str(bindings),
            str(TYPECHECK_TESTS),
            env=env,
        )
        check_isolated_imports(bindings, env)
        run(
            sys.executable,
            "-m",
            "unittest",
            "discover",
            "-s",
            str(RUNTIME_TESTS),
            "-p",
            "test_*.py",
            env=env,
        )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check-only",
        action="store_true",
        help="Check bindings already generated in python-tests/binding",
    )
    args = parser.parse_args()

    if not args.check_only:
        shutil.rmtree(BINDINGS, ignore_errors=True)
        env = os.environ.copy()
        env["RUST_PY_MODELS_EXPORT_DIR"] = str(BINDINGS)
        run("cargo", "test", "--workspace", "--all-features", "--locked", env=env)
    check(BINDINGS)


if __name__ == "__main__":
    main()
