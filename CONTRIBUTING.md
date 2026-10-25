# Contributing to Sigi

## Project context

- Implementation language: C++20
- Compiler output: generated C source; `--run` invokes a host C compiler
- Default branch: `main`

## Development setup

A C++20 compiler, CMake, Python 3, and a host C compiler are needed. Catch2 is fetched by CMake during configuration.

```sh
cmake -S . -B build -DCMAKE_BUILD_TYPE=Debug
cmake --build build -j2
ctest --test-dir build --output-on-failure
```

## Working rules

- Read the README, CMake build definition, CI workflow, and compiler-status document before changing the implementation.
- Preserve the lexer → parser → semantic analysis → C code generation pipeline.
- Unsupported language constructs must be rejected clearly; never silently drop source tokens or AST operations.
- Add or update an end-to-end test when changing syntax or generated-code behaviour.
- Keep the README, compiler-status document, and capability matrix aligned with verified C++ behaviour.
- Keep changes focused and use a feature branch and pull request.
- Do not commit generated build directories or compiler outputs.
