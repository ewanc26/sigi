# Sigi

A pure symbolic stack language that compiles to C or can be interpreted.

**Character set**: `!@#$%^&*()-+=[]{}|;:'",.<>/?\~` plus digits `0-9`

All syntax is symbolic. No alphanumeric keywords. User-defined names are prefixed with `.`.

> 🧶 Also available on [Tangled](https://tangled.org/ewancroft.uk/sigi)

---

## Installation

```sh
pip install -e .
```

This installs the `sigic` compiler.

---

## Usage

```sh
# Start interactive REPL
sigic

# Compile to C
sigic hello.si -o hello.c

# Compile and run immediately
sigic hello.si --run

# Run using the reference interpreter (faster startup)
sigic hello.si --interpret

# Debug: show tokens or AST
sigic hello.si --emit-tokens
sigic hello.si --emit-ast
```

---

## Symbol reference

| Symbol | Name | Effect |
|--------|------|--------|
| `!N` | PUSH | Push number N (int or float) |
| `@` | DUP | Duplicate top of stack |
| `#` | SWAP | Swap top two elements |
| `$` | DROP | Discard top of stack |
| `+` | ADD | Pop b, pop a → push a + b |
| `-` | SUB | Pop b, pop a → push a - b |
| `*` | MUL | Pop b, pop a → push a * b |
| `/` | DIV | Pop b, pop a → push a / b |
| `%` | MOD | Pop b, pop a → push fmod(a, b) |
| `=` | EQ | Pop b, pop a → push 1 if equal, else 0 |
| `<` | LT | Pop b, pop a → push 1 if a < b |
| `>` | GT | Pop b, pop a → push 1 if a > b |
| `~` | NOT | Pop a → push 1 if zero, else 0 |
| `|` | PRINT | Pop and print as number |
| `^` | PRINTC | Pop and print as character |
| `?` | INPUT | Read number from stdin |
| `:` | STORE | Pop address, pop value → store to var |
| `:.name`| STOREN | Pop value → store to named variable |
| `.name` | LOADN | Push value of named variable or call function |
| `<n>` | LOAD | Push value of variable n (digits 0-99) |
| `[ body ]` | WHILE | Loop while stack top is nonzero |
| `{ then ; else }` | IF-ELSE | Pop condition, execute then or else |
| `{N body }` | FUNC | Define function N (0-99) |
| `{.name body }`| FUNCN | Define named function |
| `(N)` | CALL | Call function N |
| `(.name)`| CALLN | Call named function |
| `"text"` | STRING | Print characters |
| `'x` | CHAR | Push character code |
| `\\` | COMMENT | Line comment |
| `/* body */` | BLOCK COMMENT | Multi-line comment |
| `S` | SIN | Pop a → push sin(a) |
| `C` | COS | Pop a → push cos(a) |
| `T` | TAN | Pop a → push tan(a) |
| `R` | SQRT | Pop a → push sqrt(a) |
| `P` | POW | Pop exp, pop base → push pow(base, exp) |
| `F` | FLOOR | Pop a → push floor(a) |
| `L` | LOG | Pop a → push log(a) |
| `E` | EXP | Pop a → push exp(a) |
| `M` | ABS | Pop a → push fabs(a) |
| `N` | ATAN2 | Pop y, pop x → push atan2(y, x) |
| `W` | RAND | Push random 0.0-1.0 |
| `X` | EXIT | Pop code → exit program |
| `Z` | TIME | Push current time in seconds |
| `_` | AINIT | Pop id, pop size → initialize array |
| `A` | ALOAD | Pop idx, pop id → push array[idx] |
| `a` | ASTORE | Pop idx, pop id, pop val → array[idx] = val |
| `U` | USLEEP | Pop microseconds → sleep |

---

## Robustness Features

Sigi includes several features for reliable development:

- **Static Analysis**: The compiler checks for undefined function calls and redefinitions before starting compilation or interpretation.
- **Detailed Error Reporting**: All errors (Lex, Parse, Semantic, Runtime) include line and column information, along with a source snippet and pointer.
- **Reference Interpreter**: Provides a second implementation to verify compiler behavior and enables a fast interactive workflow.
- **Automated Test Suite**: A comprehensive set of tests ensures that all features and examples remain functional.

---

## Examples

### Named Identifiers

```
{.greet "Hello!\n"}
(.greet)

!42 :.answer
.answer |  \\ prints 42
```

### Hello World

```
"Hello, World!\n"
```

### Arithmetic

```
!3 !4 + |     \ prints 7
!10 !3 - |    \ prints 7
!4 !5 * |     \ prints 20
```

### Stack operations

```
!5 @ + |      \ 10 (DUP + ADD)
!1 !2 # | |   \ 2 1 (SWAP)
```

---

## Design

Sigi is deliberately minimal. Every operation is a single character. There are no reserved words—only punctuation.

The language is stack-based with postfix notation, making parsing trivial.

Inspired by Forth, Joy, and other concatenative languages.
