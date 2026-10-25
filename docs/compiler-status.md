# C++ compiler status

This document describes the implementation in the current `main` branch, not the intended full language design.

## Pipeline

`sigic` is a C++20 program built by CMake. Its current pipeline is:

1. `src/lexer/lexer.cpp` converts source text to tokens.
2. `src/parser/parser.cpp` turns the supported token sequence into `Program.mainOps`.
3. `src/sema/sema.cpp` is currently only a placeholder; it does not yet perform meaningful semantic validation.
4. `src/codegen/codegen.cpp` lowers supported operations into C with a fixed `double stack[10000]` and `double vars[100]` runtime model.
5. `sigic --run` writes generated C into a unique temporary directory, invokes the selected host C compiler, runs the output, and removes the temporary directory.

The current frontend and backend are incomplete. Unsupported source characters/tokens and unsupported AST operations fail with a diagnostic rather than being treated as no-ops.

## Operations currently lowered end-to-end

The source and CLI smoke tests currently cover these basics:

- Numeric literals, string literals, and character literals.
- Stack manipulation: `@` (duplicate), `#` (swap), and `$` (drop).
- Arithmetic: `+`, `-`, `*`, `/`, and `%`.
- Comparisons: `=`, `<`, and `>`.
- Logical NOT: `~`.
- Numbered variables: `<n>` to load and `:` to store.
- Numeric output `|`, character output `^`, and numeric input `?`.

This is a source-inspected implementation boundary, not a claim of C conformance or full language completeness. Generated C now checks stack underflow and overflow for stack operations, rejects division and remainder by zero, and diagnoses failed numeric input. Load indices are checked during parsing and stores check the numbered-variable range at runtime.

## Not yet implemented end-to-end

The README's broad symbol reference includes syntax intended for the language but not currently lowered correctly by the C++ implementation. Basic `while` loops (`[...]`), if-else blocks (`{ then ; else }`), and loop `break` (`<>`) and `continue` (`w`) are now lowered to C and covered by end-to-end tests. Still unsupported are function definitions/calls, do-while/for loops, named variables, math built-ins, arrays, file I/O, foreign-function calls, pointer operations, aggregates, casts, switch, and goto. The C++ parser rejects unmatched delimiters, missing if-else separators, and break outside a loop. These constructs must not be advertised as usable until the parser, semantic analysis, and code generator all handle them and regression tests prove the behaviour.

The shared `capabilities.json` matrix still needs to be reconciled against the C++ implementation; see [issue #16](https://github.com/ewanc26/sigi/issues/16). Do not use its current entries as independent proof that a feature works.

The current test suite includes unit tests and a CLI smoke test. Broader parity/conformance testing and semantic checks remain tracked in [issue #16](https://github.com/ewanc26/sigi/issues/16).

## Build and test

```sh
cmake -S . -B build -DCMAKE_BUILD_TYPE=Debug
cmake --build build -j2
ctest --test-dir build --output-on-failure
```

The CLI smoke test needs Python 3 and a host C compiler. The build fetches Catch2 via CMake FetchContent.
