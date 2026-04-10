"""Sigi AST - a pure symbolic stack language."""

from dataclasses import dataclass
from typing import List, Optional, Tuple, Union


# Operations are just tuples: (opcode, optional_value)
# This keeps the AST minimal and transparent

Op = Tuple[str, Optional[float]]


@dataclass
class Function:
    """A function definition: (N) ops ;"""
    number: int
    body: List[Op]


@dataclass
class Program:
    """Complete Sigi program."""
    functions: List[Function]
    main_code: List[Op]
