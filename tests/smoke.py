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

        fractional_store = temp / "fractional store.si"
        fractional_store.write_text("!42 !1.5 :", encoding="utf-8")
        result = run([compiler, str(fractional_store), "--run", "--cc", str(wrapper)], expected_status=1)
        if "variable index out of range" not in result.stderr:
            raise AssertionError(f"fractional variable index was not diagnosed: {result.stderr!r}")

        invalid_character = temp / "invalid character.si"
        invalid_character.write_text("!999999999999999999999999999999999999 ^", encoding="utf-8")
        result = run([compiler, str(invalid_character), "--run", "--cc", str(wrapper)], expected_status=1)
        if "character value out of range" not in result.stderr:
            raise AssertionError(f"out-of-range character value was not diagnosed: {result.stderr!r}")

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

        true_branch = temp / "if true.si"
        true_branch.write_text('!1 { "then" ; "else" }', encoding="utf-8")
        result = run([compiler, str(true_branch), "--run", "--cc", str(wrapper)], expected_status=0)
        if result.stdout != "then":
            raise AssertionError(f"true if branch failed: {result.stdout!r}")

        false_branch = temp / "if false.si"
        false_branch.write_text('!0 { "then" ; "else" }', encoding="utf-8")
        result = run([compiler, str(false_branch), "--run", "--cc", str(wrapper)], expected_status=0)
        if result.stdout != "else":
            raise AssertionError(f"false if branch failed: {result.stdout!r}")

        loop = temp / "while loop.si"
        loop.write_text("!1 [ $ !0 ]", encoding="utf-8")
        result = run([compiler, str(loop), "--run", "--cc", str(wrapper)], expected_status=0)
        if result.stdout:
            raise AssertionError(f"while loop unexpectedly printed output: {result.stdout!r}")

        break_loop = temp / "break loop.si"
        break_loop.write_text('!1 [ "once" <> !0 ]', encoding="utf-8")
        result = run([compiler, str(break_loop), "--run", "--cc", str(wrapper)], expected_status=0)
        if result.stdout != "once":
            raise AssertionError(f"loop break failed: {result.stdout!r}")

        continue_loop = temp / "continue loop.si"
        continue_loop.write_text('!1 [ $ !0 w "bad" ]', encoding="utf-8")
        result = run([compiler, str(continue_loop), "--run", "--cc", str(wrapper)], expected_status=0)
        if result.stdout:
            raise AssertionError(f"continue did not skip the rest of the loop body: {result.stdout!r}")

        invalid_continue = temp / "continue outside loop.si"
        invalid_continue.write_text("w", encoding="utf-8")
        failure = run([compiler, str(invalid_continue), "-o", str(output_c)], expected_status=1)
        if "continue used outside a loop" not in failure.stderr:
            raise AssertionError(f"invalid continue was not diagnosed: {failure.stderr!r}")

        invalid_control = temp / "break outside loop.si"
        invalid_control.write_text("<>", encoding="utf-8")
        failure = run([compiler, str(invalid_control), "-o", str(output_c)], expected_status=1)
        if "break used outside a loop" not in failure.stderr:
            raise AssertionError(f"invalid break was not diagnosed: {failure.stderr!r}")

        malformed_if = temp / "malformed if.si"
        malformed_if.write_text('{ "then" }', encoding="utf-8")
        failure = run([compiler, str(malformed_if), "-o", str(output_c)], expected_status=1)
        if "unmatched" not in failure.stderr and "separator" not in failure.stderr:
            raise AssertionError(f"malformed conditional was not diagnosed: {failure.stderr!r}")

        missing = temp / "missing source.si"
        failure = run([compiler, str(missing), "-o", str(output_c)], expected_status=1)
        if "cannot read source file" not in failure.stderr:
            raise AssertionError(f"missing-source diagnostic was unclear: {failure.stderr!r}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
