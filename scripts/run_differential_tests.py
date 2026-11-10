#!/usr/bin/env python3
"""Differential tests: sigi programs versus equivalent C programs.

Usage: scripts/run_differential_tests.py [--binary SIGIC] [--cc CC] [--strict]

Each NAME.si has a NAME.c expressing the same operations. Both are run and
their stdout and exit codes must match exactly. Cases map to rows of the
shared capability matrix; by default only cases whose row is 'implemented'
for sigi are checked, the rest are reported as SKIP with the matrix status.
--strict checks every case regardless of the matrix.
"""

import argparse
import json
import pathlib
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
HERE = ROOT / "tests" / "differential"
C_FLAGS = ["-std=c99", "-Wall", "-Wextra", "-Werror", "-pedantic"]

# differential case -> capability row (None: composite, depends on missing rows)
CASE_ROW = {
    "arithmetic": "expr.operators",
    "array_bounds": "types.fixed-arrays",
    "arrays": "types.fixed-arrays",
    "bitwise": "expr.operators",
    "casts_sized": "expr.casts",
    "comparison": "expr.operators",
    "console_output": "io.console-output",
    "control_break_continue": "control.break-continue",
    "control_do_while": "control.do-while",
    "control_goto": "control.goto",
    "control_switch": "control.switch",
    "exit_status": "vars.assignment",
    "expr_comma": "expr.comma",
    "expr_compound_assign": "expr.compound-assign",
    "expr_incdec": "expr.incdec",
    "expr_ternary": "expr.ternary",
    "ffi_call": "ffi.call-c",
    "for_loop": "control.while-for",
    "func_pointers": "func.pointers",
    "func_recursion": "func.recursion",
    "functions": "func.params-returns",
    "if_else": "control.if-else",
    "integer_division": "expr.operators",
    "literals_escapes": "literals.escapes",
    "literals_numeric": "literals.numeric",
    "literals_typed": None,  # needs the typed-literal/declaration syntax, not just literals
    "logical": "types.integer-boolean",
    "program_calc_dispatch": None,
    "program_grid_paths": None,
    "program_record_sort": None,
    "strings": "types.strings",
    "types_char_literals": "types.char-literals",
    "types_enums": "types.enums",
    "types_floating": "types.floating",
    "types_initializers": "types.initializers",
    "types_multidim": "types.multidim-arrays",
    "types_pointers": "types.pointers",
    "types_qualifiers": "types.storage-qualifiers",
    "types_structs": "types.structs",
    "types_typedef": "types.typedef",
    "types_unions": "types.unions",
    "variables": "vars.assignment",
    "vars_declare_before_use": "vars.assignment",
    "while_loop": "control.while-for",
}


def run(cmd, **kwargs):
    # Byte mode: comparisons must be exact even for non-UTF-8 output.
    return subprocess.run(cmd, capture_output=True, timeout=60, **kwargs)


def preview(data: bytes) -> str:
    return data.decode("utf-8", errors="replace")


def load_cells() -> dict:
    path = ROOT / "capabilities.json"
    if not path.exists():
        return {}
    matrix = json.loads(path.read_text(encoding="utf-8"))
    return {row["id"]: row["cells"].get("sigi", {}) for row in matrix["rows"]}


def check(source: pathlib.Path, binary: str, cc: str, tmp: pathlib.Path) -> str:
    si = run([binary, str(source), "--run", "--cc", cc], cwd=ROOT)
    c_src = source.with_suffix(".c")
    c_bin = tmp / f"{source.stem}_ref"
    build = run([cc, *C_FLAGS, str(c_src), "-lm", "-o", str(c_bin)])
    if build.returncode != 0:
        print(f"FAIL {source.stem}: C reference did not build\n{preview(build.stderr)}")
        return "fail"
    ref = run([str(c_bin)])
    if si.stdout != ref.stdout or si.returncode != ref.returncode:
        print(f"FAIL {source.stem}: stdout or exit code differs")
        print(
            f"  sigi: {si.returncode} {preview(si.stdout)!r} {preview(si.stderr).strip()}"
        )
        print(f"  C:    {ref.returncode} {preview(ref.stdout)!r}")
        return "fail"
    print(f"PASS {source.stem}")
    return "pass"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", default=str(ROOT / "build" / "sigic"))
    parser.add_argument("--cc", default="cc")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="check every case, ignoring the capability matrix",
    )
    args = parser.parse_args()

    cells = load_cells()
    sources = sorted(HERE.glob("*.si"))
    if not sources:
        print("no differential cases found", file=sys.stderr)
        raise SystemExit(1)

    passed = failed = skipped = 0
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = pathlib.Path(tmpdir)
        for source in sources:
            row = CASE_ROW.get(source.stem)
            status = cells.get(row, {}).get("status") if row else None
            if not args.strict and (row is None or status != "implemented"):
                why = (
                    "no capability row for this case"
                    if row is None
                    else f"row {row} is {status}"
                )
                print(f"SKIP {source.stem} ({why})")
                skipped += 1
                continue
            if check(source, args.binary, args.cc, tmp) == "pass":
                passed += 1
            else:
                failed += 1
    print(f"{passed} passed, {skipped} skipped, {failed} failed")
    if failed:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
