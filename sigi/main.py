"""Sigi compiler main entry point."""

import argparse
import subprocess
import sys
import tempfile
from pathlib import Path

from .lexer import Lexer
from .parser import Parser
from .codegen_c import generate_c


def compile_source(source: str) -> str:
    """Compile Sigi source to C code."""
    tokens = Lexer(source).tokenize()
    program = Parser(tokens).parse()
    return generate_c(program)


def main():
    parser = argparse.ArgumentParser(
        prog="sigic",
        description="Sigi compiler - symbolic esoteric language"
    )
    parser.add_argument("source", help="Sigi source file (.si)")
    parser.add_argument("-o", "--output", help="Output C file")
    parser.add_argument("--run", action="store_true", help="Compile and run immediately")
    parser.add_argument("--cc", default="gcc", help="C compiler to use")
    parser.add_argument("--emit-tokens", action="store_true", help="Print token stream")
    parser.add_argument("--emit-ast", action="store_true", help="Print parsed operations")

    args = parser.parse_args()

    source = Path(args.source).read_text()

    # Token stream debug
    if args.emit_tokens:
        tokens = Lexer(source).tokenize()
        for tok in tokens:
            print(tok)
        return

    # AST debug
    if args.emit_ast:
        tokens = Lexer(source).tokenize()
        program = Parser(tokens).parse()
        for fn in program.functions:
            print(f"Function {fn.number}:")
            for op in fn.body:
                print(f"  {op}")
        print("Main:")
        for op in program.main_code:
            print(f"  {op}")
        return

    # Compile
    c_code = compile_source(source)

    if args.output:
        Path(args.output).write_text(c_code)
        return

    if args.run:
        with tempfile.TemporaryDirectory() as tmpdir:
            c_file = Path(tmpdir) / "out.c"
            exe_file = Path(tmpdir) / "out"
            c_file.write_text(c_code)
            result = subprocess.run(
                [args.cc, str(c_file), "-o", str(exe_file), "-lm"],
                capture_output=True,
                text=True
            )
            if result.returncode != 0:
                print(f"Compilation failed:\n{result.stderr}", file=sys.stderr)
                sys.exit(1)
            result = subprocess.run([str(exe_file)], capture_output=False)
            sys.exit(result.returncode)

    # Default: print C code
    print(c_code)


if __name__ == "__main__":
    main()
