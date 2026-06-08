"""Sigi parser - symbolic stack language parser."""

from typing import List, Optional

from .ast import (BlockOp, CallOp, Function, IfElseOp, Op, Program, PushOp,
                  SimpleOp, StoreNamedOp, StringOp, VarOp, WhileOp)
from .lexer import LexError, Token


class ParseError(Exception):
    def __init__(self, message: str, line: int, col: int):
        self.message = message
        self.line = line
        self.col = col
        super().__init__(self.message)

    def __str__(self):
        return f"{self.message} at {self.line}:{self.col}"


class Parser:
    """Sigi parser."""

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
            tok = self._current()
            raise ParseError(msg, tok.line, tok.col)
        return self._advance()

    def parse(self) -> Program:
        """Parse the token stream into a Program."""
        functions: List[Function] = []
        main_code: List[Op] = []

        while not self._match("EOF"):
            if self._match("BLOCK") and self._peek().kind in ("VAR", "IDENT"):
                functions.append(self._parse_function())
                continue

            main_code.append(self._parse_op())

        return Program(functions, main_code)

    def _parse_function(self) -> Function:
        """Parse a function definition: { N ops } or { .name ops }"""
        tok = self._advance()  # consume BLOCK
        line, col = tok.line, tok.col

        num_tok = self._current()
        if num_tok.kind not in ("NUM", "VAR", "IDENT"):
            raise ParseError(
                "Expected function name or number after '{'", num_tok.line, num_tok.col
            )

        name = num_tok.value
        if num_tok.kind in ("NUM", "VAR"):
            name = int(name)
            if name < 0 or name > 99:
                raise ParseError(
                    "Function number must be 0-99", num_tok.line, num_tok.col
                )

        self._advance()

        body = self._parse_ops()
        self._consume("ENDB", "Expected '}' to end function")

        if self._match("ELSE"):
            self._advance()

        return Function(name, body, line, col)

    def _parse_ops(self) -> List[Op]:
        """Parse a sequence of operations."""
        ops: List[Op] = []
        while not self._match("EOF", "ENDB", "WEND", "ELSE"):
            ops.append(self._parse_op())
        return ops

    def _parse_op(self) -> Op:
        """Parse a single operation."""
        tok = self._current()

        if tok.kind == "ELSE":
            self._advance()
            return SimpleOp(line=tok.line, col=tok.col, kind="NOP")

        if tok.kind == "NUM":
            self._advance()
            return PushOp(line=tok.line, col=tok.col, value=tok.value)

        if tok.kind == "VAR":
            self._advance()
            return VarOp(line=tok.line, col=tok.col, name=int(tok.value))

        if tok.kind == "IDENT":
            self._advance()
            return VarOp(line=tok.line, col=tok.col, name=tok.value)

        if tok.kind == "STORE_IDENT":
            self._advance()
            return StoreNamedOp(line=tok.line, col=tok.col, name=tok.value)

        if tok.kind == "CALL":
            self._advance()
            num_tok = self._current()
            if num_tok.kind not in ("NUM", "VAR", "IDENT"):
                raise ParseError(
                    "Expected function name or number after '('",
                    num_tok.line,
                    num_tok.col,
                )

            target = num_tok.value
            if num_tok.kind in ("NUM", "VAR"):
                target = int(target)

            self._advance()
            self._consume("ENDCALL", "Expected ')' after function identifier")
            return CallOp(line=tok.line, col=tok.col, target=target)

        if tok.kind == "STRING":
            self._advance()
            return StringOp(line=tok.line, col=tok.col, value=tok.value)

        if tok.kind == "CHAR":
            self._advance()
            return PushOp(line=tok.line, col=tok.col, value=float(tok.value))

        if tok.kind == "WHILE":
            self._advance()
            body = self._parse_ops()
            self._consume("WEND", "Expected ']' to end while loop")
            return WhileOp(line=tok.line, col=tok.col, body=body)

        if tok.kind == "BLOCK":
            return self._parse_block()

        if tok.kind == "ELSE":
            raise ParseError("Unexpected ';'", tok.line, tok.col)

        self._advance()

        op_map = {
            "DUP": "DUP",
            "SWAP": "SWAP",
            "DROP": "DROP",
            "ADD": "ADD",
            "SUB": "SUB",
            "MUL": "MUL",
            "DIV": "DIV",
            "MOD": "MOD",
            "EQ": "EQ",
            "LT": "LT",
            "GT": "GT",
            "NOT": "NOT",
            "PRINT": "PRINT",
            "PRINTC": "PRINTC",
            "INPUT": "INPUT",
            "STORE": "STORE",
            "SIN": "SIN",
            "COS": "COS",
            "TAN": "TAN",
            "SQRT": "SQRT",
            "POW": "POW",
            "FLOOR": "FLOOR",
            "LOG": "LOG",
            "EXP": "EXP",
            "ABS": "ABS",
            "ATAN2": "ATAN2",
            "RAND": "RAND",
            "EXIT": "EXIT",
            "TIME": "TIME",
            "ALEN": "ALEN",
            "ALOAD": "ALOAD",
            "ASTORE": "ASTORE",
            "AINIT": "AINIT",
            "USLEEP": "USLEEP",
        }

        if tok.kind in op_map:
            return SimpleOp(line=tok.line, col=tok.col, kind=op_map[tok.kind])

        raise ParseError(f"Unexpected token {tok.kind}", tok.line, tok.col)

    def _parse_block(self) -> Op:
        """Parse a block with optional else: { ops } or { ops ; ops }"""
        tok = self._advance()  # consume BLOCK
        line, col = tok.line, tok.col

        then_ops = self._parse_ops()

        if self._match("ELSE"):
            self._advance()
            else_ops = self._parse_ops()
            self._consume("ENDB", "Expected '}' after else block")
            return IfElseOp(line=line, col=col, then_body=then_ops, else_body=else_ops)
        else:
            self._consume("ENDB", "Expected '}' to end block")
            return BlockOp(line=line, col=col, body=then_ops)


def from_source(source: str) -> Program:
    """Parse source string into Program."""
    from .lexer import Lexer

    tokens = Lexer(source).tokenize()
    return Parser(tokens).parse()
