# Sigi

A pure symbolic stack language that compiles to C.

**Character set**: `!@#$%^&*()-+=[]{}|;:'",.<>/?\~` plus digits `0-9`

All syntax is symbolic. No alphanumeric keywords.

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
# Compile to C
sigic hello.si -o hello.c

# Compile and run immediately
sigic hello.si --run

# Debug: show tokens
sigic hello.si --emit-tokens

# Debug: show AST
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
| `<n>` | LOAD | Push value of variable n (digits 0-99) |
| `[ body ]` | WHILE | Loop while stack top is nonzero |
| `{ then ; else }` | IF-ELSE | Pop condition, execute then or else |
| `{N body }` | FUNC | Define function N (0-99) |
| `(N)` | CALL | Call function N |
| `"text"` | STRING | Print characters |
| `'x` | CHAR | Push character code |
| `\\` | COMMENT | Line comment |
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

## Examples

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

### Variables

```
!42 !0 :       \ store 42 in var 0
0 |            \ print 42
```

### Functions

```
{0 "Hi\n"}     \ define fn 0
{1 !42 |}      \ define fn 1
(0)            \ call fn 0
(1)            \ call fn 1
```

### Control flow

```
!1 { "yes" ; "no" }    \ prints "yes"
!0 { "yes" ; "no" }    \ prints "no"
```

### While loops

```
!5 [ @ | !1 - ] $      \\ prints 5 4 3 2 1
```

### Trigonometry

```
!0 S |                 \\ prints 0 (sin(0))
!1.570796 S |          \\ prints 1 (sin(π/2))
!0 C |                 \\ prints 1 (cos(0))
```

### Arrays

```
!100 !0 _              \\ initialize array 0 with 100 elements
!42 !0 !5 a            \\ array[5] = 42
!0 !5 A |              \\ prints 42 (array[5])
```

### Math functions

```
!2 R |                 \\ prints 1.414... (sqrt(2))
!2 !3 P |              \\ prints 8 (2^3)
!-5 M |                \\ prints 5 (abs)
W |                    \\ prints random 0.0-1.0
```

### Timed animation

```
!1000000 U             \\ sleep for 1 second
```

---

## Design

Sigi is deliberately minimal. Every operation is a single character. There are no reserved words—only punctuation.

The language is stack-based with postfix notation, making parsing trivial.

Inspired by Forth, Joy, and other concatenative languages.
