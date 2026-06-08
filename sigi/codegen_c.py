"""Sigi C code generator."""

import re
from typing import List, Set, Union

from .ast import (BlockOp, CallOp, Function, IfElseOp, Op, Program, PushOp,
                  SimpleOp, StoreNamedOp, StringOp, VarOp, WhileOp)


class CodegenError(Exception):
    pass


def _c_ident(name: Union[int, str]) -> str:
    """Convert Sigi identifier to safe C identifier."""
    if isinstance(name, int):
        return f"{name}"
    return re.sub(r"\W+", "_", name)


def generate_c(program: Program) -> str:
    """Generate C code from a Sigi Program."""
    lines = []

    # Prelude
    lines.append("#include <stdio.h>")
    lines.append("#include <stdlib.h>")
    lines.append("#include <math.h>")
    lines.append("#include <time.h>")
    lines.append("#include <unistd.h>")
    lines.append("")
    lines.append("#define STACK_SIZE 1000")
    lines.append("#define MAX_ARRAYS 10")
    lines.append("")
    lines.append("static double stack[STACK_SIZE];")
    lines.append("static int sp = 0;")
    lines.append("static double vars[100];")

    # Collect all named variables
    named_vars: Set[str] = set()

    def collect_vars(ops: List[Op]):
        for op in ops:
            if isinstance(op, (VarOp, StoreNamedOp)) and isinstance(op.name, str):
                named_vars.add(op.name)
            elif isinstance(op, (WhileOp, BlockOp)):
                collect_vars(op.body)
            elif isinstance(op, IfElseOp):
                collect_vars(op.then_body)
                collect_vars(op.else_body)

    collect_vars(program.main_code)
    for fn in program.functions:
        collect_vars(fn.body)

    for name in sorted(named_vars):
        lines.append(f"static double var_{_c_ident(name)} = 0;")

    lines.append("static double *arrays[MAX_ARRAYS];")
    lines.append("static int array_sizes[MAX_ARRAYS];")
    lines.append("")
    lines.append("static void arr_init(int id, int size) {")
    lines.append(
        '    if (id < 0 || id >= MAX_ARRAYS) { fprintf(stderr, "Array ID out of range\\n"); exit(1); }'
    )
    lines.append("    arrays[id] = (double *)calloc(size, sizeof(double));")
    lines.append("    array_sizes[id] = size;")
    lines.append("}")
    lines.append("")
    lines.append("static void arr_free(int id) {")
    lines.append(
        '    if (id < 0 || id >= MAX_ARRAYS) { fprintf(stderr, "Array ID out of range\\n"); exit(1); }'
    )
    lines.append("    if (arrays[id]) { free(arrays[id]); arrays[id] = NULL; array_sizes[id] = 0; }")
    lines.append("}")
    lines.append("")
    lines.append("static void push(double x) {")
    lines.append(
        '    if (sp >= STACK_SIZE) { fprintf(stderr, "Stack overflow\\n"); exit(1); }'
    )
    lines.append("    stack[sp++] = x;")
    lines.append("}")
    lines.append("")
    lines.append("static double pop(void) {")
    lines.append('    if (sp <= 0) { fprintf(stderr, "Stack underflow\\n"); exit(1); }')
    lines.append("    return stack[--sp];")
    lines.append("}")
    lines.append("")

    # Forward declare functions
    for fn in program.functions:
        lines.append(f"static void func_{_c_ident(fn.name)}(void);")
    lines.append("")

    # Function definitions
    for fn in program.functions:
        lines.append(f"static void func_{_c_ident(fn.name)}(void) {{")
        for op in fn.body:
            lines.extend(_codegen_op(op, indent=1))
        lines.append("}")
        lines.append("")

    # Main function
    lines.append("int main(void) {")
    lines.append("    srand((unsigned)time(NULL));")
    for op in program.main_code:
        lines.extend(_codegen_op(op, indent=1))
    lines.append("    return 0;")
    lines.append("}")

    return "\n".join(lines)


def _codegen_op(op: Op, indent: int = 0) -> List[str]:
    """Generate C code for a single operation."""
    prefix = "    " * indent
    lines = []

    if isinstance(op, PushOp):
        if op.value == int(op.value):
            lines.append(f"{prefix}push({int(op.value)});")
        else:
            lines.append(f"{prefix}push({op.value});")
        return lines

    if isinstance(op, VarOp):
        if isinstance(op.name, int):
            lines.append(f"{prefix}push(vars[{op.name}]);")
        else:
            lines.append(f"{prefix}push(var_{_c_ident(op.name)});")
        return lines

    if isinstance(op, StoreNamedOp):
        lines.append(f"{prefix}var_{_c_ident(op.name)} = pop();")
        return lines

    if isinstance(op, StringOp):
        for ch in op.value:
            lines.append(f"{prefix}putchar({ord(ch)});")
        return lines

    if isinstance(op, CallOp):
        lines.append(f"{prefix}func_{_c_ident(op.target)}();")
        return lines

    if isinstance(op, WhileOp):
        lines.append(f"{prefix}while (1) {{")
        lines.append(f"{prefix}    if (sp <= 0) break;")
        lines.append(f"{prefix}    double _cond = stack[sp - 1];")
        lines.append(f"{prefix}    if (_cond == 0.0) break;")
        for body_op in op.body:
            lines.extend(_codegen_op(body_op, indent + 1))
        lines.append(f"{prefix}}}")
        return lines

    if isinstance(op, BlockOp):
        lines.append(f"{prefix}{{")
        for body_op in op.body:
            lines.extend(_codegen_op(body_op, indent + 1))
        lines.append(f"{prefix}}}")
        return lines

    if isinstance(op, IfElseOp):
        lines.append(f"{prefix}{{")
        lines.append(f"{prefix}    double _cond = pop();")
        lines.append(f"{prefix}    if (_cond != 0.0) {{")
        for body_op in op.then_body:
            lines.extend(_codegen_op(body_op, indent + 2))
        lines.append(f"{prefix}    }} else {{")
        for body_op in op.else_body:
            lines.extend(_codegen_op(body_op, indent + 2))
        lines.append(f"{prefix}    }}")
        lines.append(f"{prefix}}}")
        return lines

    if not isinstance(op, SimpleOp):
        raise CodegenError(f"Unexpected AST node: {type(op)}")

    code = op.kind

    # Binary operations
    binops = {"ADD": "+", "SUB": "-", "MUL": "*", "DIV": "/"}
    if code in binops:
        lines.append(
            f"{prefix}{{ double b = pop(); double a = pop(); push(a {binops[code]} b); }}"
        )
        return lines

    if code == "MOD":
        lines.append(
            f"{prefix}{{ double b = pop(); double a = pop(); push(fmod(a, b)); }}"
        )
        return lines

    comparisons = {"EQ": "==", "LT": "<", "GT": ">"}
    if code in comparisons:
        lines.append(
            f"{prefix}{{ double b = pop(); double a = pop(); push((a {comparisons[code]} b) ? 1.0 : 0.0); }}"
        )
        return lines

    if code == "NOP":
        return lines

    if code == "NOT":
        lines.append(f"{prefix}push((pop() == 0.0) ? 1.0 : 0.0);")
        return lines

    if code == "DUP":
        lines.append(f"{prefix}{{ double x = stack[sp - 1]; push(x); }}")
        return lines

    if code == "SWAP":
        lines.append(
            f"{prefix}{{ double t = stack[sp - 1]; stack[sp - 1] = stack[sp - 2]; stack[sp - 2] = t; }}"
        )
        return lines

    if code == "DROP":
        lines.append(f"{prefix}(void)pop();")
        return lines

    # Functions
    math_fns = {
        "SIN": "sin",
        "COS": "cos",
        "TAN": "tan",
        "SQRT": "sqrt",
        "FLOOR": "floor",
        "LOG": "log",
        "EXP": "exp",
        "ABS": "fabs",
    }
    if code in math_fns:
        lines.append(f"{prefix}push({math_fns[code]}(pop()));")
        return lines

    if code == "POW":
        lines.append(
            f"{prefix}{{ double e = pop(); double b = pop(); push(pow(b, e)); }}"
        )
        return lines

    if code == "ATAN2":
        lines.append(
            f"{prefix}{{ double y = pop(); double x = pop(); push(atan2(y, x)); }}"
        )
        return lines

    if code == "RAND":
        lines.append(f"{prefix}push((double)rand() / RAND_MAX);")
        return lines

    if code == "EXIT":
        lines.append(f"{prefix}exit((int)pop());")
        return lines

    if code == "TIME":
        lines.append(f"{prefix}push((double)time(NULL));")
        return lines

    if code == "ALEN":
        lines.append(f"{prefix}{{ int id = (int)pop(); push(array_sizes[id]); }}")
        return lines

    if code == "ALOAD":
        lines.append(
            f"{prefix}{{ int idx = (int)pop(); int id = (int)pop(); push(arrays[id][idx]); }}"
        )
        return lines

    if code == "ASTORE":
        lines.append(
            f"{prefix}{{ int idx = (int)pop(); int id = (int)pop(); double val = pop(); arrays[id][idx] = val; }}"
        )
        return lines

    if code == "AINIT":
        lines.append(f"{prefix}{{ int id = (int)pop(); int size = (int)pop(); arr_init(id, size); }}")
        return lines

    if code == "AFREE":
        lines.append(f"{prefix}{{ int id = (int)pop(); arr_free(id); }}")
        return lines

    if code == "USLEEP":

        lines.append(f"{prefix}usleep((useconds_t)pop());")
        return lines

    if code == "PRINT":
        lines.append(f'{prefix}printf("%g\\n", pop());')
        return lines

    if code == "PRINTC":
        lines.append(f"{prefix}putchar((int)pop());")
        return lines

    if code == "INPUT":
        lines.append(f'{prefix}{{ double x; scanf("%lf", &x); push(x); }}')
        return lines

    if code == "STORE":
        lines.append(
            f"{prefix}{{ int idx = (int)pop(); double val = pop(); vars[idx] = val; }}"
        )
        return lines

    raise CodegenError(f"Unknown opcode kind: {code}")
