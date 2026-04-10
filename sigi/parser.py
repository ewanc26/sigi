"""Sigi parser - symbolic stack language parser."""

from typing import List, Optional, Tuple

from .ast import Op, Function, Program
from .lexer import Token, LexError


class ParseError(Exception):
    pass


class Parser:
    """
    Sigi parser.

    Grammar:
        program   -> top_level* EOF
        top_level -> function | op
        function  -> BLOCK <number> ops ENDB
        ops       -> op*
        op        -> NUM | VAR | CALL NUM ENDCALL | CHAR | STRING
                   | WHILE ops WEND | BLOCK ops else_part? ENDB
                   | STORE | PRINT | PRINTC | INPUT | ELSE
                   | DUP | SWAP | DROP | ADD | SUB | MUL | DIV | MOD
                   | EQ | LT | GT | NOT | NEG | ABS | SQRT | ROUND
        else_part -> ELSE ops
    """

    def __init__(self, tokens: List[Token]):
        self.tokens = tokens
        self.pos = 0

    def _current(self) -> Token:
        return self.tokens[self.pos]

    def _peek(self, offset: int = 1) -> Token:
        idx = self.pos + offset
        if idx >= len(self.tokens):
            return self.tokens[-1]  # EOF
        return self.tokens[idx]

    def _advance(self) -> Token:
        tok = self._current()
        if tok.kind != "EOF":
            self.pos += 1
        return tok

    def _match(self, *kinds: str) -> bool:
        return self._current().kind in kinds

    def _consume(self, kind: str, msg: str) -> Token:
        if self._current().kind != kind:
            raise ParseError(f"{msg} at {self._current().line}:{self._current().col}; got {self._current().kind}")
        return self._advance()

    def parse(self) -> Program:
        """Parse the token stream into a Program."""
        functions: List[Function] = []
        main_code: List[Op] = []

        while not self._match("EOF"):
            # Check for function definition: BLOCK followed by VAR (digit)
            # NOT NUM (which is from !N push syntax)
            if self._match("BLOCK") and self._peek().kind == "VAR":
                functions.append(self._parse_function())
                continue

            main_code.append(self._parse_op())

        return Program(functions, main_code)

    def _parse_function(self) -> Function:
        """Parse a function definition: { N ops }"""
        self._advance()  # consume BLOCK

        # Get function number (can be VAR for 0-9, or NUM for larger)
        num_tok = self._current()
        if num_tok.kind not in ("NUM", "VAR"):
            raise ParseError(f"Expected function number after '{{' at {num_tok.line}:{num_tok.col}")
        fn_num = int(num_tok.value)
        if fn_num < 0 or fn_num > 99:
            raise ParseError(f"Function number must be 0-99 at {num_tok.line}:{num_tok.col}")
        self._advance()

        body = self._parse_ops()

        self._consume("ENDB", "Expected '}' to end function")

        # Optionally consume stray ELSE after function def
        if self._match("ELSE"):
            self._advance()

        return Function(fn_num, body)

    def _parse_ops(self) -> List[Op]:
        """Parse a sequence of operations."""
        ops: List[Op] = []
        while not self._match("EOF", "ENDB", "WEND", "ELSE"):
            ops.append(self._parse_op())
        return ops

    def _parse_op(self) -> Op:
        """Parse a single operation."""
        tok = self._current()

        # Skip stray ELSE tokens at top level (they can appear after function defs)
        if tok.kind == "ELSE":
            self._advance()
            return ("NOP", None)

        # Push number
        if tok.kind == "NUM":
            self._advance()
            return ("NUM", tok.value)

        # Push variable value
        if tok.kind == "VAR":
            self._advance()
            return ("VAR", float(tok.value))

        # Function call: ( N )
        if tok.kind == "CALL":
            self._advance()
            num_tok = self._current()
            if num_tok.kind not in ("NUM", "VAR"):
                raise ParseError(f"Expected function number after '(' at {num_tok.line}:{num_tok.col}")
            fn_num = int(num_tok.value) if num_tok.kind == "NUM" else int(num_tok.value)
            self._advance()
            self._consume("ENDCALL", "Expected ')' after function number")
            return ("CALL", float(fn_num))

        # String literal
        if tok.kind == "STRING":
            self._advance()
            return ("STRING", tok.value)

        # Character literal
        if tok.kind == "CHAR":
            self._advance()
            return ("NUM", float(tok.value))

        # While loop: [ ops ]
        if tok.kind == "WHILE":
            self._advance()
            body = self._parse_ops()
            self._consume("WEND", "Expected ']' to end while loop")
            return ("WHILE", None, body)

        # Block / If-Else: { ops } or { ops ; ops }
        if tok.kind == "BLOCK":
            return self._parse_block()

        # Else shouldn't appear outside block
        if tok.kind == "ELSE":
            raise ParseError(f"Unexpected ';' at {tok.line}:{tok.col}")

        self._advance()

        # Map single-char tokens to operations
        op_map = {
            "DUP": ("DUP", None),
            "SWAP": ("SWAP", None),
            "DROP": ("DROP", None),
            "ADD": ("ADD", None),
            "SUB": ("SUB", None),
            "MUL": ("MUL", None),
            "DIV": ("DIV", None),
            "MOD": ("MOD", None),
            "EQ": ("EQ", None),
            "LT": ("LT", None),
            "GT": ("GT", None),
            "NOT": ("NOT", None),
            "PRINT": ("PRINT", None),
            "PRINTC": ("PRINTC", None),
            "INPUT": ("INPUT", None),
            "STORE": ("STORE", None),
        }

        if tok.kind in op_map:
            return op_map[tok.kind]

        raise ParseError(f"Unexpected token {tok.kind} at {tok.line}:{tok.col}")

    def _parse_block(self) -> Tuple[str, None, List[Op], List[Op]]:
        """Parse a block with optional else: { ops } or { ops ; ops }"""
        self._advance()  # consume BLOCK

        then_ops = self._parse_ops()

        if self._match("ELSE"):
            self._advance()
            else_ops = self._parse_ops()
            self._consume("ENDB", "Expected '}' after else block")
            return ("IFELSE", None, then_ops, else_ops)
        else:
            self._consume("ENDB", "Expected '}' to end block")
            return ("BLOCK", None, then_ops)


def from_source(source: str) -> Program:
    """Parse source string into Program."""
    from .lexer import Lexer
    tokens = Lexer(source).tokenize()
    return Parser(tokens).parse()
