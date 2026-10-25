# Sigi

A pure symbolic stack language that compiles to C or runs in an interpreter.

**Character set:** `!@#$%^&*()-+=[]{}|;:'",.<>/?\~` plus digits `0-9`

All syntax is symbolic. No alphanumeric keywords. User-defined names are prefixed with `.`.

> Also available on [Tangled](https://tangled.org/ewancroft.uk/sigi)

## Install

Requires a Rust toolchain (`cargo`).

```sh
cargo install --path .
```

This installs the `sigic` compiler into `~/.cargo/bin`.

## Usage

```sh
# Compile to C, written to stdout
sigic hello.si

# Compile to a C file
sigic hello.si -o hello.c

# Compile and run (uses gcc; override with --cc)
sigic hello.si --run
sigic hello.si --run --cc clang
```

The REPL (`sigic` with no arguments, or `--repl`) and `--interpret` are not implemented in the Rust compiler yet and currently print a message and exit.

## Symbol reference

| Symbol | Name | Effect |
|--------|------|--------|
| `!N` | PUSH | Push number N: decimal, `0x` hex, `0b` binary, leading-0 octal, fractions; `u`/`l`/`ll`/`ul`/`ull` suffixes push a typed integer (untyped numbers are `f64`) |
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
| `\|` | PRINT | Pop and print as number |
| `^` | PRINTC | Pop and print as character |
| `?` | INPUT | Read number from stdin |
| `:` | STORE | Pop address, pop value. A pointer stores the value through it; a number names a variable slot |
| `:.name` | STOREN | Pop value → store to named variable |
| `.name` | LOADN | Push value of named variable or call function |
| `<n>` | LOAD | Push value of variable n (digits 0-99) |
| `[ body ]` | WHILE | Loop while stack top nonzero |
| `{ then ; else }` | IF-ELSE | Pop condition, execute then or else |
| `{N body }` | FUNC | Define function N (0-99) |
| `{.name body }` | FUNCN | Define named function |
| `(N)` | CALL | Call function N |
| `(.name)` | CALLN | Call named function |
| `"text"` | STRING | Print characters; C escapes (`\n`, `\t`, `\xHH`, `\ooo`, `\uXXXX`, `\UXXXXXXXX`) decode to bytes |
| `'x` | CHAR | Push character code; accepts C escapes such as `'\n`, `'\x41`, `'\101` |
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
| `K` | AFREE | Pop id → free array |
| `O` | FILE_OPEN | Pop mode, pop len, pop chars → fd |
| `G` | FILE_READ | Pop fd, pop size → push bytes, read_len |
| `H` | FILE_WRITE | Pop fd, pop size, pop data → bytes_written |
| `Y` | FILE_CLOSE | Pop fd → close |
| `A` | ALOAD | Pop idx, pop id → push array[idx] |
| `a` | ASTORE | Pop idx, pop id, pop val → array[idx] = val |
| `U` | USLEEP | Pop microseconds → sleep |
| `B` | BITAND | Pop b, pop a → push a & b (integers) |
| `V` | BITOR | Pop b, pop a → push a \| b (integers) |
| `Q` | BITXOR | Pop b, pop a → push a ^ b (integers) |
| `J` | BITNOT | Pop a → push ~a (integer) |
| `D` | SHL | Pop b, pop a → push a << b (b in 0-63) |
| `I` | SHR | Pop b, pop a → push a >> b (b in 0-63) |
| `b` | LOGAND | Pop b, pop a → push 1 if both nonzero, else 0 |
| `v` | LOGOR | Pop b, pop a → push 1 if either nonzero, else 0 |
| `l` | INTDIV | Pop b, pop a → push a / b with C integer division |
| `f{ body }` | FOR | Pop count, run body that many times |
| `x "header" .name !N` | FFI_DECL | Declare C function `name` with N double arguments from `<header>`; a `.c` path is compiled and linked from the source's directory |
| `c .name` | FFI_CALL | Pop N arguments, call declared C function, push result |
| `d[ body ]` | DO-WHILE | Run body, then repeat while stack top nonzero |
| `q` | BREAK | Leave the innermost loop or switch |
| `w` | CONTINUE | Continue the innermost loop |
| `e{ .A .B .C !5 .D }` | ENUM | Declare integer constants; each takes the previous value + 1 unless followed by `!N` |
| `h.name` | LABEL | Define a goto label in the current function |
| `g.name` | GOTO | Jump to a label in the same function |
| `s{ k!n body m body }` | SWITCH | Pop selector; `k!n` starts a case, `m` the default; arms fall through as in C |
| `o.name` / `o<n>` | FUNC_REF | Push the index of a function |
| `p` | CALL_PTR | Pop a function index and call that function |
| `i.T` | CAST | Pop a, push it converted to type T (`f32`, `f64`, `ld`, `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`); integer casts wrap, out-of-range float casts exit 1. `ld` is C `long double`, whose width is the target's |
| `j.name` / `j<n>` | ADDRESS | Push a pointer to a named or numbered variable cell |
| `y` | DEREF | Pop a pointer, push the value it points to |
| `z` | SIZEOF | Pop a, push its size in bytes (`u64`) |
| `z.T` | SIZEOF_TYPE | Push the size in bytes of type T |
| `z.Name` | SIZEOF_AGG | Push the size in bytes of struct Name |
| `r.Name{ .f .T ... }` | DECL | Define struct Name; each member is `.name` then a scalar type or an earlier struct |
| `r.Name .obj` | DECL | Declare a struct object named obj |
| `r.Name .obj = { ... }` | DECL | Declare a struct object, filling members in order from a brace list; nested braces fill struct members, and missing members are zeroed |
| `u.Name{ .f .T ... }` | UNION | Define union Name; members share storage, each starting at offset 0 |
| `n.f` | MEMBER | Pop a pointer to a struct, push a pointer to its member f |
| `t.Name .T` | TYPEDEF | Declare `Name` as an alias for scalar type T (top level only) |

## Error handling

- Checks for undefined function calls and redefinitions before compiling
- All errors (Lex, Parse, Semantic, Runtime) include line, column, source snippet, and pointer
- Reference interpreter provides a second implementation for verification

## Examples

### Named identifiers

```
{.greet "Hello!\n"}
(.greet)

!42 :.answer
.answer |  \\ prints 42
```

### File I/O

```
!116 !101 !115 !116 !46 !116 !120 !116 !8 !1 O :.my_fd
!116 !101 !115 !116 !4 .my_fd H $
.my_fd Y
```

## Design

Every operation is a single character. No reserved words — only punctuation. Stack-based with postfix notation, making parsing trivial.

Inspired by Forth, Joy, and other concatenative languages.

## Support

If you find this project useful, consider supporting its development:

[![Ko-fi](https://img.shields.io/badge/Ko--fi-F16061?style=for-the-badge&logo=ko-fi&logoColor=white)](https://ko-fi.com/ewancroft)
[![GitHub Sponsors](https://img.shields.io/badge/GitHub%20Sponsors-30363D?style=for-the-badge&logo=github&logoColor=white)](https://github.com/sponsors/ewanc26)
