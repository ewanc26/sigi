"""Sigi AST - a pure symbolic stack language."""

from dataclasses import dataclass, field
from typing import List, Optional, Union


@dataclass
class Node:
    line: int
    col: int


@dataclass
class Op(Node):
    pass


@dataclass
class PushOp(Op):
    value: float


@dataclass
class VarOp(Op):
    name: Union[int, str]


@dataclass
class CallOp(Op):
    target: Union[int, str]


@dataclass
class StoreNamedOp(Op):
    name: str


@dataclass
class StringOp(Op):
    value: str


@dataclass
class BlockOp(Op):
    body: List[Op] = field(default_factory=list)


@dataclass
class WhileOp(Op):
    body: List[Op] = field(default_factory=list)


@dataclass
class IfElseOp(Op):
    then_body: List[Op] = field(default_factory=list)
    else_body: List[Op] = field(default_factory=list)


@dataclass
class SimpleOp(Op):
    kind: str


@dataclass
class Function:
    name: Union[int, str]
    body: List[Op]
    line: int = 0
    col: int = 0


class SemanticError(Exception):
    def __init__(self, message: str, line: int = 0, col: int = 0):
        self.message = message
        self.line = line
        self.col = col
        super().__init__(self.message)

    def __str__(self):
        if self.line:
            return f"{self.message} at {self.line}:{self.col}"
        return self.message


@dataclass
class Program:
    functions: List[Function]
    main_code: List[Op]

    def validate(self):
        """Perform semantic analysis on the program."""
        defined_fns = {f.name for f in self.functions}

        def check_ops(ops: List[Op]):
            for op in ops:
                if isinstance(op, CallOp):
                    if op.target not in defined_fns:
                        raise SemanticError(
                            f"Call to undefined function ({op.target})",
                            op.line,
                            op.col,
                        )
                elif isinstance(op, (WhileOp, BlockOp)):
                    check_ops(op.body)
                elif isinstance(op, IfElseOp):
                    check_ops(op.then_body)
                    check_ops(op.else_body)

        for fn in self.functions:
            check_ops(fn.body)
        check_ops(self.main_code)
