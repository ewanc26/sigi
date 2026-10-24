use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn twin_cc() -> String {
    std::env::var("SIGI_TWIN_CC").unwrap_or_else(|_| "cc".to_string())
}

fn cases() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/differential");
    let mut sources: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("tests/differential exists")
        .map(|entry| entry.expect("readable entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "si"))
        .collect();
    sources.sort();
    assert!(!sources.is_empty(), "no differential cases found");
    sources
}

#[test]
fn sigi_matches_c_twins() {
    let cc = twin_cc();
    let scratch = tempfile::tempdir().expect("scratch dir");

    for source in cases() {
        let name = source
            .file_stem()
            .expect("file stem")
            .to_string_lossy()
            .into_owned();
        let twin_src = source.with_extension("c");
        let twin_bin = scratch.path().join(format!("{name}_twin"));

        let build = Command::new(&cc)
            .args(["-std=c23", "-Wall", "-Wextra", "-Werror", "-pedantic"])
            .arg(&twin_src)
            .arg("-o")
            .arg(&twin_bin)
            .arg("-lm")
            .status()
            .expect("run C compiler on twin");
        assert!(build.success(), "{name}: C twin failed to build");

        let expected = Command::new(&twin_bin)
            .output()
            .expect("run C twin");

        let actual = Command::new(env!("CARGO_BIN_EXE_sigic"))
            .arg(&source)
            .args(["--run", "--cc", &cc])
            .output()
            .expect("run sigic");

        assert_eq!(
            (
                String::from_utf8_lossy(&actual.stdout).into_owned(),
                actual.status.code()
            ),
            (
                String::from_utf8_lossy(&expected.stdout).into_owned(),
                expected.status.code()
            ),
            "{name}: sigi diverges from its C twin (stdout, exit status)"
        );
    }
}
