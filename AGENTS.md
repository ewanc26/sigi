# AGENTS.md

Guidance for agents working on Sigi, a pure symbolic stack language implemented in Rust with both interpreter and C compiler backends.

## Architecture

- `src/` owns lexing, parsing, AST/IR, interpreter, C code generation, CLI, and REPL.
- `runtime/` supports generated programs.
- `examples/` and `.si` files define expected language behavior.

## Invariants

- Compiler and interpreter must agree on every symbol's stack effect, numeric behavior, identifiers, I/O, and errors.
- Preserve source spans in diagnostics and reject stack underflow/invalid arity before unsafe execution.
- Generated C must escape source data, avoid undefined behavior, and be deterministic.
- `--run` must use safe temporary files and argument-based process execution.
- REPL state should persist intentionally between entries and recover after an error.

## Validation

Run `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, and `cargo build --release`. Differential-test examples through interpreter and compiled backends, including errors, named identifiers, file I/O, numeric edges, nested constructs, REPL recovery, and compiler failure. Do not commit `target/`, egg-info changes, generated C, or binaries unless explicitly part of a fixture.
