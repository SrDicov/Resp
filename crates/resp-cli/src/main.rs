use clap::Parser;
use resp_parser::parse_programo;
use resp_transpiler::transpile;
use std::fs;
use std::path::PathBuf;
#[derive(Parser)]
#[command(name = "resp", version, about = "Resp - Esperanto to Rust transpiler")]
struct Cli {
    #[arg(short = 'o')]
    output: Option<PathBuf>,

    #[arg(long)]
    compile: bool,

    #[arg(long)]
    fmt: bool,

    file: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let source = fs::read_to_string(&cli.file)
        .expect("Failed to read source file");

    let programo = parse_programo(&source).unwrap_or_else(|e| {
        eprintln!("Parse error: {}", e);
        std::process::exit(1);
    });

    let rust_tokens = transpile(&programo);
    let rust_source = rust_tokens.to_string();

    let output_path = cli.output.unwrap_or_else(|| {
        cli.file.with_extension("rs")
    });

    fs::write(&output_path, &rust_source)
        .expect("Failed to write output file");

    println!("Transpiled to {}", output_path.display());

    if cli.fmt {
        let status = std::process::Command::new("rustfmt")
            .arg(&output_path)
            .status()
            .expect("Failed to run rustfmt");
        if !status.success() {
            eprintln!("Warning: rustfmt failed");
        }
    }

    if cli.compile {
        let status = std::process::Command::new("rustc")
            .arg(&output_path)
            .arg("-o")
            .arg(output_path.with_extension(""))
            .status()
            .expect("Failed to run rustc");
        if !status.success() {
            std::process::exit(1);
        }
        println!("Compiled to {}", output_path.with_extension("").display());
    }
}
