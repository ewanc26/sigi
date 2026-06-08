use clap::{Parser};
use std::fs;
use std::path::PathBuf;
use sigi::lexer::Lexer;
use sigi::parser::Parser as SigiParser;
use sigi::codegen::Codegen;

#[derive(Parser)]
#[command(name = "sigic", about = "Sigi compiler - symbolic esoteric language")]
struct Cli {
    source: Option<PathBuf>,
    
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    #[arg(long)]
    run: bool,
    
    #[arg(long)]
    interpret: bool,
    
    #[arg(long)]
    repl: bool,
    
    #[arg(long, default_value = "gcc")]
    cc: String,
}

fn main() {
    let cli = Cli::parse();

    if cli.repl || (cli.source.is_none() && std::env::args().len() == 1) {
        println!("Sigi REPL not implemented in Rust yet. Using Python version...");
        // Fallback to Python REPL if needed, or implement it here.
        return;
    }

    let source_path = cli.source.expect("Source file required");
    let source = fs::read_to_string(source_path).expect("Failed to read source file");

    let lexer = Lexer::new(&source);
    let mut parser = SigiParser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse");
    
    if cli.interpret {
        println!("Interpreter not implemented in Rust yet. Using Python version...");
        return;
    }

    let codegen = Codegen::new(program);
    let c_code = codegen.generate();

    if let Some(out_path) = cli.output {
        fs::write(out_path, c_code).expect("Failed to write output");
    } else if cli.run {
        use std::process::Command;
        use std::io::Write;
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
        println!("{}", c_code);
    }
}
