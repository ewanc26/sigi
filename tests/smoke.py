#!/usr/bin/env python3
"""End-to-end smoke tests for the currently implemented C++ compiler path."""

import shlex
import subprocess
import sys
import tempfile
from pathlib import Path


def run(command: list[str], *, expected_status: int) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(command, text=True, capture_output=True, timeout=20)
    if result.returncode != expected_status:
        raise AssertionError(
            f"command {command!r} returned {result.returncode}, expected {expected_status}\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    return result


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: smoke.py <sigic> <host-c-compiler>", file=sys.stderr)
        return 2

    compiler = str(Path(sys.argv[1]).resolve())
    host_cc = str(Path(sys.argv[2]).resolve())
    root = Path(__file__).resolve().parents[1]
    example = str(root / "examples" / "hello_world.si")

    # Spaces and an apostrophe exercise shell quoting for compiler and temp paths.
    with tempfile.TemporaryDirectory(prefix="sigi smoke ' path ") as temporary:
        temp = Path(temporary)
        wrapper = temp / "host C compiler's wrapper"
        wrapper.write_text("#!/bin/sh\nexec " + shlex.quote(host_cc) + ' "$@"\n', encoding="utf-8")
        wrapper.chmod(0o755)

        result = run([compiler, example, "--run", "--cc", str(wrapper)], expected_status=0)
        if result.stdout != "Hello, World!\n":
            raise AssertionError(f"unexpected output: {result.stdout!r}")

        output_c = temp / "generated output with spaces.c"
        run([compiler, example, "-o", str(output_c)], expected_status=0)
        generated = output_c.read_text(encoding="utf-8")
        if "int main()" not in generated or "putchar(72)" not in generated:
            raise AssertionError("generated C does not contain the expected hello-world program")

        variables = temp / "variables.si"
        variables.write_text("!42 !1 : <1>|", encoding="utf-8")
        result = run([compiler, str(variables), "--run", "--cc", str(wrapper)], expected_status=0)
        if result.stdout != "42\n":
            raise AssertionError(f"numbered variable store/load failed: {result.stdout!r}")

        bad_store = temp / "bad store.si"
        bad_store.write_text("!42 !100 :", encoding="utf-8")
        result = run([compiler, str(bad_store), "--run", "--cc", str(wrapper)], expected_status=1)
        if "variable index out of range" not in result.stderr:
            raise AssertionError(f"out-of-range variable store was not diagnosed: {result.stderr!r}")

        underflow = temp / "stack underflow.si"
        underflow.write_text("+", encoding="utf-8")
        failure = run([compiler, str(underflow), "--run", "--cc", str(wrapper)], expected_status=1)
        if "stack underflow" not in failure.stderr:
            raise AssertionError(f"stack underflow was not diagnosed: {failure.stderr!r}")

        overflow = temp / "stack overflow.si"
        overflow.write_text("!1 " * 10001, encoding="utf-8")
        failure = run([compiler, str(overflow), "--run", "--cc", str(wrapper)], expected_status=1)
        if "stack overflow" not in failure.stderr:
            raise AssertionError(f"stack overflow was not diagnosed: {failure.stderr!r}")

        divide_by_zero = temp / "division by zero.si"
        divide_by_zero.write_text("!1 !0 /", encoding="utf-8")
        failure = run([compiler, str(divide_by_zero), "--run", "--cc", str(wrapper)], expected_status=1)
        if "division by zero" not in failure.stderr:
            raise AssertionError(f"division by zero was not diagnosed: {failure.stderr!r}")

        missing = temp / "missing source.si"
        failure = run([compiler, str(missing), "-o", str(output_c)], expected_status=1)
        if "cannot read source file" not in failure.stderr:
            raise AssertionError(f"missing-source diagnostic was unclear: {failure.stderr!r}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
