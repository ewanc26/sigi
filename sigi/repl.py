"""Sigi REPL - interactive environment."""

import sys

from .ast import SemanticError
from .interpreter import Interpreter, RuntimeError
from .lexer import Lexer, LexError
from .parser import ParseError, Parser


def start_repl():
    """Start the Sigi interactive shell."""
    interpreter = Interpreter()
    print("Sigi REPL v1.0")
    print("Type symbols and press Enter. 'EXIT' or Ctrl-D to quit.")

    while True:
        try:
            line = input("si> ")
            if not line:
                continue

            # Special case for exit if they type it (not the symbol)
            if line.strip().upper() == "EXIT":
                break

            tokens = Lexer(line).tokenize()
            # Remove EOF for incremental parsing if possible,
            # but Sigi parser expects EOF.
            # For REPL, we can just parse the line as main_code.
            program = Parser(tokens).parse()

            # Merge new functions into interpreter
            for fn in program.functions:
                interpreter.functions[fn.name] = fn

            # Execute main code from this line
            interpreter.run(program.main_code)

            # Print stack if not empty
            if interpreter.stack:
                stack_str = " ".join(
                    str(int(x) if x == int(x) else x) for x in interpreter.stack
                )
                print(f"stack: [{stack_str}]")

        except EOFError:
            print("\nGoodbye!")
            break
        except (LexError, ParseError, SemanticError, RuntimeError) as e:
            # We don't have the full source for format_error here easily,
            # but we can print the error.
            print(f"Error: {e}")
        except Exception as e:
            print(f"Unexpected error: {e}")
