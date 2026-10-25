//! Regression tests for the lexer and parser decisions that are not covered
//! by the differential C twins: rejected forms must fail with a clear message,
//! and symbols that are also literal letters must keep their operator meaning.

use std::process::{Command, Output};

fn cc() -> String {
    std::env::var("SIGI_TWIN_CC").unwrap_or_else(|_| "cc".to_string())
}

/// Compile and run a source snippet with sigic, in its own temp directory.
fn run_source(source: &str) -> Output {
    let dir = tempfile::tempdir().expect("scratch dir");
    let path = dir.path().join("snippet.si");
    std::fs::write(&path, source).expect("write snippet");
    Command::new(env!("CARGO_BIN_EXE_sigic"))
        .arg(&path)
        .args(["--run", "--cc", &cc()])
        .output()
        .expect("run sigic")
}

fn assert_rejected(source: &str, message: &str) {
    let out = run_source(source);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{source:?} should exit 1");
    assert!(
        stderr.contains(message),
        "{source:?}: stderr {stderr:?} does not contain {message:?}"
    );
}

#[test]
fn reference_before_array_declaration_is_a_clear_error() {
    // C rejects `blocked` used before its declaration, so sigi does too, and
    // names the required order instead of reporting a duplicate definition.
    assert_rejected(
        "{.p j.blocked y | }\nr .f64 [ !25 ] .blocked\n",
        "before its declaration",
    );
}

#[test]
fn reference_after_array_declaration_compiles() {
    let out = run_source(
        "r .f64 [ !2 ] .blocked\n!1.5 j.blocked !1 + :\n{.p j.blocked !1 + y | }\n(.p)\n",
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1.5\n");
}

#[test]
fn unknown_escape_is_rejected() {
    // `\e` is a GNU extension, not a C23 escape; the C twins would not build.
    assert_rejected("\"a\\e\"", "Unknown escape sequence");
}

#[test]
fn octal_constant_with_eight_or_nine_is_rejected() {
    assert_rejected("!08 |", "octal");
}

#[test]
fn radix_prefix_without_digits_is_rejected() {
    assert_rejected("!0x |", "Expected number");
}

#[test]
fn uppercase_letters_keep_their_operator_meaning() {
    // `L` is LOG, so `!2L` is push 2 then log. It must not become a literal.
    let out = run_source("!2L |\n");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0.693147\n");

    // `X` is EXIT, so `!0X10` is push 0 then exit with status 0.
    let out = run_source("!0X10 |\n");
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty());
}
