"""Sigi compiler main entry point."""

import argparse
import subprocess
import sys
import tempfile
from pathlib import Path

from .ast import SemanticError
from .codegen_c import generate_c
from .interpreter import Interpreter, RuntimeError
from .lexer import Lexer, LexError
from .parser import ParseError, Parser
from .repl import start_repl


def format_error(source: str, message: str, line: int, col: int) -> str:
    """Format error message with source context."""
    if line <= 0:
        return f"Error: {message}"

    lines = source.splitlines()
    if line > len(lines):
        return f"Error: {message} at {line}:{col}"

    error_line = lines[line - 1]
    pointer = " " * (col - 1) + "^"
    return (
        f"Error: {message}\n"
        f"  at line {line}, column {col}:\n"
        f"    {error_line}\n"
        f"    {pointer}"
    )


def compile_source(source: str) -> str:
    """Compile Sigi source to C code."""
    try:
        tokens = Lexer(source).tokenize()
        program = Parser(tokens).parse()
        program.validate()
        return generate_c(program)
    except (LexError, ParseError, SemanticError) as e:
        line, col = 0, 0
        if hasattr(e, "line"):
            line, col = e.line, e.col
        msg = str(e).split(" at ")[0]
        print(format_error(source, msg, line, col), file=sys.stderr)
        sys.exit(1)


def main():
    parser = argparse.ArgumentParser(
        prog="sigic", description="Sigi compiler - symbolic esoteric language"
    )
    parser.add_argument("source", nargs="?", help="Sigi source file (.si)")
    parser.add_argument("-o", "--output", help="Output C file")
    parser.add_argument(
        "--run", action="store_true", help="Compile and run immediately"
    )
    parser.add_argument(
        "--interpret", action="store_true", help="Use interpreter instead of C compiler"
    )
    parser.add_argument("--repl", action="store_true", help="Start interactive REPL")
    parser.add_argument("--cc", default="gcc", help="C compiler to use")
    parser.add_argument("--emit-tokens", action="store_true", help="Print token stream")
    parser.add_argument(
        "--emit-ast", action="store_true", help="Print parsed operations"
    )

    args = parser.parse_args()

    # REPL mode
    if args.repl or (not args.source and len(sys.argv) == 1):
        start_repl()
        return

    if not args.source:
        parser.print_help()
        return

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
            print(f"Function {fn.name}:")
            for op in fn.body:
                print(f"  {op}")
        print("Main:")
        for op in program.main_code:
            print(f"  {op}")
        return

    # Interpreter mode
    if args.interpret:
        try:
            tokens = Lexer(source).tokenize()
            program = Parser(tokens).parse()
            program.validate()
            Interpreter(program).run(program.main_code)
            return
        except (LexError, ParseError, SemanticError, RuntimeError) as e:
            line, col = 0, 0
            if hasattr(e, "line"):
                line, col = e.line, e.col
            msg = str(e).split(" at ")[0]
            print(format_error(source, msg, line, col), file=sys.stderr)
            sys.exit(1)

    # Compile mode
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
                text=True,
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
