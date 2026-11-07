# AGENTS.md

Guidance for AI coding agents working in **sigi**.

## Project overview



- Language: C++
- Default branch: main

## Working rules

- Inspect the README, manifests, CI workflows, and nearby code before editing.
- Preserve existing architecture, naming, formatting, and error-handling conventions.
- Use project scripts for validation; never claim checks you did not run.
- Keep changes scoped and update tests or documentation when behavior changes.
- Use feature branches and pull requests.
- Treat generated files, credentials, deployment configuration, and release metadata as sensitive.

## Recent history

Document exponent-form floating-point literals; Remove unsupported generated-C CLI flag from smoke test; Document exponent-form numeric literal support; Test exponent-form floats and checked character conversion; Recognize exponent-only decimal literals as floating-point numbers; Document checked runtime conversions in README; Pass the expected status to generated-C smoke test; Verify character output uses checked conversion helper