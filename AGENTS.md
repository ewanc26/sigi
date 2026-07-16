# AGENTS.md

Guidance for agents working on Sigi, a Rust compiler for a symbolic stack language. Treat source as authority where the README describes functionality that does not exist.

## Current compiler

- `src/lexer.rs` tokenizes prefixed numbers, numbered/named variables, strings/chars, comments, compound delimiters, math, arrays, file I/O, sleep, time, random, and exit operations with line/column locations.
- `parser.rs` produces a flat AST plus function definitions. It currently masks lexer errors as EOF at location 0:0 and has no separate semantic pass for undefined/redefined functions despite README claims.
- `codegen.rs` embeds `runtime/prelude.c`, emits static named variables and C99 functions, lowers stack/flow/math/file/array operations, and links `--run` output with `-lm`. The runtime has a fixed 1,000-value stack, 100 numeric variables, and 10 dynamic arrays.
- `src/main.rs` is compile-only. Invoking without a source, `--repl`, or `--interpret` prints a “not implemented” message and returns; there is no Python implementation in this repository. The README's `pip install -e .`, interpreter, REPL, token dump, and AST dump instructions are stale.
- The CLI reads/parses/writes with `expect`/`unwrap`. `--output` takes precedence over `--run`; compiler failure and executed-program status are not propagated as the `sigic` exit status.

## Invariants

- Keep lexer, parser, codegen, runtime prelude, README symbol table, and examples synchronized. File-I/O comments in `examples/file_io.si` are stale; source token mappings are authoritative.
- Preserve stack operand order and loop semantics (while inspects but does not pop the top condition). Validate division/modulo, array reinitialization/size, file descriptors, Unicode-to-byte conversion, and C identifier collisions deliberately.
- Propagate structured lex/parse/semantic/runtime diagnostics instead of adding more panics. Invoke compilers via argument arrays and return their failures.
- `Cargo.lock` is ignored today; do not claim locked application builds.

## Validation

Run `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, and `cargo build --release`. Existing automated coverage is minimal, so compile every `.si` example, run safe examples, inspect emitted C, and add tests for every token/compound form, malformed input, source locations, undefined/redefined calls, named C-identifier collisions, stack/array/file bounds, compiler-not-found/failure, and child exit propagation. Do not claim interpreter differential testing until an interpreter exists.
