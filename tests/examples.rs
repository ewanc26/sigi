use std::path::PathBuf;
use std::process::Command;

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(format!("{name}.si"))
}

fn run_example(name: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_sigic"))
        .arg(example(name))
        .args(["--run", "--cc", "cc"])
        .output()
        .expect("failed to run sigic")
}

#[test]
fn hello_world_prints_greeting() {
    let out = run_example("hello_world");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "Hello, World!\n");
}

#[test]
fn factorial_prints_120() {
    let out = run_example("factorial");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "120");
}

#[test]
fn undefined_function_fails_to_build() {
    let out = run_example("undefined_fn");
    assert!(!out.status.success());
}
