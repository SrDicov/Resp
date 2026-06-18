use clap::Parser;
use resp_parser::parse_programo;
use resp_sema::analyze_program;
use resp_transpiler::transpile_with_sema;
use std::collections::HashMap;
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

    let mut programo = parse_programo(&source).unwrap_or_else(|e| {
        eprintln!("Parse error: {}", e);
        std::process::exit(1);
    });

    // Run semantic analysis (populates SymbolTable, applies name mangling)
    let tabla = analyze_program(&mut programo);

    // Generate sourcemap during transpilation
    let rust_tokens = transpile_with_sema(&programo, &tabla);
    let rust_source = rust_tokens.to_string();

    let output_path = cli.output.unwrap_or_else(|| {
        cli.file.with_extension("rs")
    });

    // Write sourcemap
    let sourcemap_path = output_path.with_extension("sourcemap.json");
    // Build a simple line mapping based on the transpiled output
    let resp_lines: Vec<&str> = source.lines().collect();
    let rs_lines: Vec<&str> = rust_source.lines().collect();
    let mut map = HashMap::new();
    // Approximate mapping: each transpiled line maps to the corresponding .resp line group
    let ratio = if !rs_lines.is_empty() { (resp_lines.len() as f64 / rs_lines.len() as f64).max(1.0) } else { 1.0 };
    for (i, _line) in rs_lines.iter().enumerate() {
        let resp_line = ((i as f64) * ratio).min((resp_lines.len() - 1) as f64) as usize + 1;
        map.insert(i + 1, resp_line);
    }
    let sourcemap = serde_json::to_string_pretty(&map).unwrap_or_default();
    let _ = fs::write(&sourcemap_path, &sourcemap);

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
        let child = std::process::Command::new("rustc")
            .arg(&output_path)
            .arg("-o")
            .arg(output_path.with_extension(""))
            .arg("--error-format=json")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to run rustc");

        let output = child.wait_with_output().expect("Failed to wait for rustc");
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Try to parse JSON diagnostics and remap to .resp lines
            let remapped = remap_errors(&stderr, &sourcemap_path);
            eprintln!("{}", remapped);
            std::process::exit(1);
        }
        println!("Compiled to {}", output_path.with_extension("").display());
    }
}

fn remap_errors(stderr: &str, sourcemap_path: &PathBuf) -> String {
    let map_content = fs::read_to_string(sourcemap_path).unwrap_or_default();
    let line_map: HashMap<usize, usize> =
        serde_json::from_str(&map_content).unwrap_or_default();
    let mut result = String::new();

    for line in stderr.lines() {
        if line.starts_with('{') {
            // Try to parse JSON diagnostic
            if let Ok(diag) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(rendered) = diag.get("rendered").and_then(|r| r.as_str()) {
                    // Replace .rs line references with .resp line references
                    let mut remapped = rendered.to_string();
                    for (rs_line, resp_line) in &line_map {
                        let rs_str = format!(":{}:{}", rs_line, "");
                        let resp_str = format!(" (resp line {})", resp_line);
                        remapped = remapped.replace(&rs_str, &resp_str);
                    }
                    result.push_str(&remapped);
                    result.push('\n');
                }
            } else {
                result.push_str(line);
                result.push('\n');
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}
