# Contributing to Sigi

A pure symbolic stack language with a C++20 compiler frontend that emits C source, where sigic in `--run` mode invokes a host C compiler to build and run the generated code. All syntax is symbolic: no alphanumeric keywords, and user-defined names are prefixed with `.`. Thanks for wanting to help!

## Getting started

```sh
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build -j
# binary: build/sigic
```

## Notes for this project

- **Purity is the concept.** Everything stays symbolic; adding an alphanumeric keyword should be treated as a language-design decision to argue for in an issue, not a convenience in a PR.
- The current compiler covers a tested core subset (numeric/string/character literals, stack manipulation, arithmetic, comparisons, `~`, I/O, numbered variables, `while` loops, if-else, `break`/`continue`); the full symbol reference describes the intended language. **Check [the C++ compiler status] docs before implementing an unsupported symbol** — implementing one is welcome, but deliberately and visibly, not accidentally.
- The compiler emits C: prefer readable, portable C99 over clever codegen, since users read the output.
- Update the symbol reference and status docs in the same PR as any behaviour change.

## Before opening a PR

- Build passes and your example programs round-trip: `sigic x.si -o x.c` then `sigic x.si --run` produce the same result.
- Conventional commits; focused PRs; include the sample programs you compiled with the change.
- AI-assisted PRs welcome; stand behind every line and run both paths yourself.

## Licence

See [LICENSE](LICENSE). Contributions accepted under the same terms.