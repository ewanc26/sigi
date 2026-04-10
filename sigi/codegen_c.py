"""Sigi C code generator."""

from typing import List

from .ast import Op, Function, Program


class CodegenError(Exception):
    pass


def generate_c(program: Program) -> str:
    """Generate C code from a Sigi Program."""
    lines = []

    # Prelude
    lines.append('#include <stdio.h>')
    lines.append('#include <stdlib.h>')
    lines.append('#include <math.h>')
    lines.append('#include <time.h>')
    lines.append('')
    lines.append('#define STACK_SIZE 1000')
    lines.append('')
    lines.append('static double stack[STACK_SIZE];')
    lines.append('static int sp = 0;')
    lines.append('static double vars[100];')
    lines.append('')
    lines.append('static void push(double x) {')
    lines.append('    if (sp >= STACK_SIZE) { fprintf(stderr, "Stack overflow\\n"); exit(1); }')
    lines.append('    stack[sp++] = x;')
    lines.append('}')
    lines.append('')
    lines.append('static double pop(void) {')
    lines.append('    if (sp <= 0) { fprintf(stderr, "Stack underflow\\n"); exit(1); }')
    lines.append('    return stack[--sp];')
    lines.append('}')
    lines.append('')

    # Forward declare functions
    for fn in program.functions:
        lines.append(f'static void func_{fn.number}(void);')
    lines.append('')

    # Function definitions
    for fn in program.functions:
        lines.append(f'static void func_{fn.number}(void) {{')
        for op in fn.body:
            lines.extend(_codegen_op(op, indent=1))
        lines.append('}')
        lines.append('')

    # Main function
    lines.append('int main(void) {')
    lines.append('    srand((unsigned)time(NULL));')
    for op in program.main_code:
        lines.extend(_codegen_op(op, indent=1))
    lines.append('    return 0;')
    lines.append('}')

    return '\n'.join(lines)


def _codegen_op(op: Op, indent: int = 0) -> List[str]:
    """Generate C code for a single operation."""
    prefix = '    ' * indent
    code, value = op[0], op[1] if len(op) > 1 else None
    lines = []

    # Simple push
    if code == "NUM":
        if op[1] == int(op[1]):
            lines.append(f'{prefix}push({int(op[1])});')
        else:
            lines.append(f'{prefix}push({op[1]});')
        return lines

    # Variable load (digits 0-99)
    if code == "VAR":
        lines.append(f'{prefix}push(vars[{int(value)}]);')
        return lines

    # String literal
    if code == "STRING":
        s = value
        for ch in s:
            lines.append(f'{prefix}putchar({ord(ch)});')
        return lines

    # Function call
    if code == "CALL":
        lines.append(f'{prefix}func_{int(value)}();')
        return lines

    # While loop [ body ]
    # Semantics: peek at stack top. If zero or empty, exit. Otherwise execute body.
    # Body has access to stack (counter on top). Body MUST push condition for next iteration.
    if code == "WHILE":
        body = op[2] if len(op) > 2 else []
        lines.append(f'{prefix}while (1) {{')
        lines.append(f'{prefix}    if (sp <= 0) break;')
        lines.append(f'{prefix}    double _cond = stack[sp - 1];')
        lines.append(f'{prefix}    if (_cond == 0.0) break;')
        # Don't pop! Body has access to stack top as loop counter
        for body_op in body:
            lines.extend(_codegen_op(body_op, indent + 1))
        lines.append(f'{prefix}}}')
        return lines

    # Block / If-Else
    if code == "BLOCK":
        then_ops = op[2] if len(op) > 2 else []
        lines.append(f'{prefix}{{')
        for body_op in then_ops:
            lines.extend(_codegen_op(body_op, indent + 1))
        lines.append(f'{prefix}}}')
        return lines

    if code == "IFELSE":
        then_ops = op[2] if len(op) > 2 else []
        else_ops = op[3] if len(op) > 3 else []
        lines.append(f'{prefix}{{')
        lines.append(f'{prefix}    double _cond = pop();')
        lines.append(f'{prefix}    if (_cond != 0.0) {{')
        for body_op in then_ops:
            lines.extend(_codegen_op(body_op, indent + 2))
        lines.append(f'{prefix}    }} else {{')
        for body_op in else_ops:
            lines.extend(_codegen_op(body_op, indent + 2))
        lines.append(f'{prefix}    }}')
        lines.append(f'{prefix}}}')
        return lines

    # Binary operations
    binops = {
        "ADD": '+',
        "SUB": '-',
        "MUL": '*',
        "DIV": '/',
    }
    if code in binops:
        lines.append(f'{prefix}{{ double b = pop(); double a = pop(); push(a {binops[code]} b); }}')
        return lines

    # Modulo (fmod)
    if code == "MOD":
        lines.append(f'{prefix}{{ double b = pop(); double a = pop(); push(fmod(a, b)); }}')
        return lines

    # Comparisons
    comparisons = {
        "EQ": '==',
        "LT": '<',
        "GT": '>',
    }
    if code in comparisons:
        lines.append(f'{prefix}{{ double b = pop(); double a = pop(); push((a {comparisons[code]} b) ? 1.0 : 0.0); }}')
        return lines

    # No-op
    if code == "NOP":
        return lines

    # Unary operations
    if code == "NOT":
        lines.append(f'{prefix}push((pop() == 0.0) ? 1.0 : 0.0);')
        return lines

    # Stack operations
    if code == "DUP":
        lines.append(f'{prefix}{{ double x = stack[sp - 1]; push(x); }}')
        return lines

    if code == "SWAP":
        lines.append(f'{prefix}{{ double t = stack[sp - 1]; stack[sp - 1] = stack[sp - 2]; stack[sp - 2] = t; }}')
        return lines

    if code == "DROP":
        lines.append(f'{prefix}(void)pop();')
        return lines

    # I/O
    if code == "PRINT":
        lines.append(f'{prefix}printf("%g\\n", pop());')
        return lines

    if code == "PRINTC":
        lines.append(f'{prefix}putchar((int)pop());')
        return lines

    if code == "INPUT":
        lines.append(f'{prefix}{{ double x; scanf("%lf", &x); push(x); }}')
        return lines

    # Variable store
    if code == "STORE":
        lines.append(f'{prefix}{{ int idx = (int)pop(); double val = pop(); vars[idx] = val; }}')
        return lines

    raise CodegenError(f"Unknown opcode: {code}")
