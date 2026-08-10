use clap::Parser;
use sigi::codegen::Codegen;
use sigi::lexer::Lexer;
use sigi::parser::Parser as SigiParser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sigic", about = "Sigi compiler - symbolic esoteric language")]
struct Cli {
    /// Path to the .si source file.
    source: Option<PathBuf>,

    /// Write generated C to this file instead of stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Compile and run the generated binary immediately.
    #[arg(long)]
    run: bool,

    /// Run via the Python reference interpreter (stub in Rust frontend).
    #[arg(long)]
    interpret: bool,

    /// Launch an interactive REPL (stub in Rust frontend).
    #[arg(long)]
    repl: bool,

    /// C compiler to use when --run is set.
    #[arg(long, default_value = "gcc")]
    cc: String,

    /// Print sponsor links and exit.
    #[arg(long)]
    support: bool,
}

fn main() {
    let cli = Cli::parse();

    // ─── Support ──────────────────────────────────────────────────

    if cli.support {
        println!("Support Sigi development:");
        println!("  Ko-fi: https://ko-fi.com/ewancroft");
        println!("  GitHub Sponsors: https://github.com/sponsors/ewanc26");
        return;
    }

    // ─── REPL / Interpreter ───────────────────────────────────────

    if cli.repl || (cli.source.is_none() && std::env::args().len() == 1) {
        // The Rust frontend is compile-only for now; REPL and interpreter
        // delegate to the Python reference implementation.
        println!("Sigi REPL not implemented in Rust yet. Using Python version...");
        return;
    }

    let source_path = cli.source.expect("Source file required");
    let source = fs::read_to_string(source_path).expect("Failed to read source file");

    // ─── Compile Pipeline: Lex → Parse → Codegen ──────────────────

    let lexer = Lexer::new(&source);
    let mut parser = SigiParser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse");

    if cli.interpret {
        println!("Interpreter not implemented in Rust yet. Using Python version...");
        return;
    }

    let codegen = Codegen::new(program);
    let c_code = codegen.generate();

    // ─── Emit or Compile-and-Run ──────────────────────────────────

    if let Some(out_path) = cli.output {
        // Write C to a user-specified file for inspection or manual compilation.
        fs::write(out_path, c_code).expect("Failed to write output");
    } else if cli.run {
        // Write C to a temp dir, compile with the chosen cc, and execute.
        // The tempfile crate ensures cleanup on drop.
        use std::process::Command;
        let tmp = tempfile::Builder::new().tempdir().unwrap();
        let c_file = tmp.path().join("out.c");
        let exe_file = tmp.path().join("out");
        fs::write(&c_file, c_code).unwrap();

        let status = Command::new(&cli.cc)
            .arg(&c_file)
            .arg("-o")
            .arg(&exe_file)
            .arg("-lm")
            .status()
            .expect("Failed to execute C compiler");

        if status.success() {
            Command::new(&exe_file).status().unwrap();
        }
    } else {
        // Default: print the generated C to stdout so it can be piped.
        println!("{}", c_code);
    }
}
