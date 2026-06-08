"""Sigi interpreter - reference implementation."""

import random
import sys
import time
from math import atan2, cos, exp, fabs, floor, fmod, log, pow, sin, sqrt, tan
from typing import Dict, List, Optional, Union

from .ast import (BlockOp, CallOp, Function, IfElseOp, Op, Program, PushOp,
                  SimpleOp, StoreNamedOp, StringOp, VarOp, WhileOp)


class RuntimeError(Exception):
    def __init__(self, message: str, line: int = 0, col: int = 0):
        self.message = message
        self.line = line
        self.col = col
        super().__init__(self.message)

    def __str__(self):
        if self.line:
            return f"{self.message} at {self.line}:{self.col}"
        return self.message


class Interpreter:
    """Sigi interpreter."""

    def __init__(self, program: Optional[Program] = None):
        self.stack: List[float] = []
        self.vars: Dict[Union[int, str], float] = {i: 0.0 for i in range(100)}
        self.arrays: Dict[int, List[float]] = {}
        self.files: Dict[int, object] = {} # Will store file objects
        self.functions: Dict[Union[int, str], Function] = {}
        if program:
            for fn in program.functions:
                self.functions[fn.name] = fn

    def push(self, val: float):
        self.stack.append(float(val))

    def pop(self, op: Op) -> float:
        if not self.stack:
            raise RuntimeError("Stack underflow", op.line, op.col)
        return self.stack.pop()

    def run(self, ops: List[Op]):
        for op in ops:
            self.execute_op(op)

    def execute_op(self, op: Op):
        if isinstance(op, PushOp):
            self.push(op.value)
        elif isinstance(op, VarOp):
            val = self.vars.get(op.name, 0.0)
            self.push(val)
        elif isinstance(op, StoreNamedOp):
            val = self.pop(op)
            self.vars[op.name] = val
        elif isinstance(op, StringOp):
            sys.stdout.write(op.value)
            sys.stdout.flush()
        elif isinstance(op, CallOp):
            fn = self.functions.get(op.target)
            if not fn:
                raise RuntimeError(f"Undefined function ({op.target})", op.line, op.col)
            self.run(fn.body)
        elif isinstance(op, WhileOp):
            while True:
                if not self.stack:
                    break
                cond = self.stack[-1]
                if cond == 0.0:
                    break
                self.run(op.body)
        elif isinstance(op, BlockOp):
            self.run(op.body)
        elif isinstance(op, IfElseOp):
            cond = self.pop(op)
            if cond != 0.0:
                self.run(op.then_body)
            else:
                self.run(op.else_body)
        elif isinstance(op, SimpleOp):
            self.execute_simple_op(op)

    def execute_simple_op(self, op: SimpleOp):
        kind = op.kind

        if kind == "ADD":
            b, a = self.pop(op), self.pop(op)
            self.push(a + b)
        elif kind == "SUB":
            b, a = self.pop(op), self.pop(op)
            self.push(a - b)
        elif kind == "MUL":
            b, a = self.pop(op), self.pop(op)
            self.push(a * b)
        elif kind == "DIV":
            b, a = self.pop(op), self.pop(op)
            if b == 0:
                raise RuntimeError("Division by zero", op.line, op.col)
            self.push(a / b)
        elif kind == "MOD":
            b, a = self.pop(op), self.pop(op)
            self.push(fmod(a, b))
        elif kind == "EQ":
            b, a = self.pop(op), self.pop(op)
            self.push(1.0 if a == b else 0.0)
        elif kind == "LT":
            b, a = self.pop(op), self.pop(op)
            self.push(1.0 if a < b else 0.0)
        elif kind == "GT":
            b, a = self.pop(op), self.pop(op)
            self.push(1.0 if a > b else 0.0)
        elif kind == "NOT":
            a = self.pop(op)
            self.push(1.0 if a == 0.0 else 0.0)
        elif kind == "DUP":
            if not self.stack:
                raise RuntimeError("Stack underflow", op.line, op.col)
            self.push(self.stack[-1])
        elif kind == "SWAP":
            if len(self.stack) < 2:
                raise RuntimeError("Stack underflow", op.line, op.col)
            self.stack[-1], self.stack[-2] = self.stack[-2], self.stack[-1]
        elif kind == "DROP":
            self.pop(op)
        elif kind == "PRINT":
            val = self.pop(op)
            if val == int(val):
                print(int(val))
            else:
                print(val)
        elif kind == "PRINTC":
            sys.stdout.write(chr(int(self.pop(op))))
            sys.stdout.flush()
        elif kind == "INPUT":
            try:
                line = sys.stdin.readline()
                self.push(float(line.strip()))
            except ValueError:
                self.push(0.0)
        elif kind == "STORE":
            idx = int(self.pop(op))
            val = self.pop(op)
            self.vars[idx] = val
        elif kind == "SIN":
            self.push(sin(self.pop(op)))
        elif kind == "COS":
            self.push(cos(self.pop(op)))
        elif kind == "TAN":
            self.push(tan(self.pop(op)))
        elif kind == "SQRT":
            self.push(sqrt(self.pop(op)))
        elif kind == "POW":
            e, b = self.pop(op), self.pop(op)
            self.push(pow(b, e))
        elif kind == "FLOOR":
            self.push(floor(self.pop(op)))
        elif kind == "LOG":
            self.push(log(self.pop(op)))
        elif kind == "EXP":
            self.push(exp(self.pop(op)))
        elif kind == "ABS":
            self.push(fabs(self.pop(op)))
        elif kind == "ATAN2":
            y, x = self.pop(op), self.pop(op)
            self.push(atan2(y, x))
        elif kind == "RAND":
            self.push(random.random())
        elif kind == "EXIT":
            sys.exit(int(self.pop(op)))
        elif kind == "TIME":
            self.push(time.time())
        elif kind == "ALEN":
            id = int(self.pop(op))
            self.push(len(self.arrays.get(id, [])))
        elif kind == "ALOAD":
            idx, id = int(self.pop(op)), int(self.pop(op))
            arr = self.arrays.get(id)
            if not arr:
                raise RuntimeError(f"Array {id} not initialized", op.line, op.col)
            if idx < 0 or idx >= len(arr):
                raise RuntimeError("Array index out of range", op.line, op.col)
            self.push(arr[idx])
        elif kind == "ASTORE":
            idx, id, val = int(self.pop(op)), int(self.pop(op)), self.pop(op)
            arr = self.arrays.get(id)
            if not arr:
                raise RuntimeError(f"Array {id} not initialized", op.line, op.col)
            if idx < 0 or idx >= len(arr):
                raise RuntimeError("Array index out of range", op.line, op.col)
            arr[idx] = val
        elif kind == "AINIT":
            id, size = int(self.pop(op)), int(self.pop(op))
            self.arrays[id] = [0.0] * size
        elif kind == "AFREE":
            id = int(self.pop(op))
            if id in self.arrays:
                del self.arrays[id]
        elif kind == "FILE_OPEN":
            mode = int(self.pop(op))
            path_len = int(self.pop(op))
            path_chars = [int(self.pop(op)) for _ in range(path_len)]
            path = "".join(chr(c) for c in path_chars)
            mode_str = "r" if mode == 0 else "w"
            f = open(path, mode_str)
            fd = f.fileno()
            self.files[fd] = f
            self.push(float(fd))
        elif kind == "FILE_READ":
            fd = int(self.pop(op))
            size = int(self.pop(op))
            f = self.files.get(fd)
            if not f: raise RuntimeError("Invalid file descriptor", op.line, op.col)
            data = f.read(size)
            for ch in reversed(data):
                self.push(float(ord(ch)))
            self.push(float(len(data)))
        elif kind == "FILE_WRITE":
            fd = int(self.pop(op))
            size = int(self.pop(op))
            data = "".join(chr(int(self.pop(op))) for _ in range(size))
            f = self.files.get(fd)
            if not f: raise RuntimeError("Invalid file descriptor", op.line, op.col)
            f.write(data)
            self.push(float(size))
        elif kind == "FILE_CLOSE":
            fd = int(self.pop(op))
            f = self.files.get(fd)
            if not f: raise RuntimeError("Invalid file descriptor", op.line, op.col)
            f.close()
            del self.files[fd]
        elif kind == "USLEEP":
            time.sleep(self.pop(op) / 1000000.0)
        elif kind == "NOP":
            pass
        else:
            raise RuntimeError(f"Unknown operation: {kind}", op.line, op.col)
