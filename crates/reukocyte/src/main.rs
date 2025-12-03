use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: reukocyte <file>...");
        return ExitCode::from(1);
    }

    let mut total_diagnostics = 0;

    for path in &args[1..] {
        match fs::read(path) {
            Ok(source) => {
                let diagnostics = check_file(path, &source);
                total_diagnostics += diagnostics;
            }
            Err(e) => {
                eprintln!("Error reading {}: {}", path, e);
            }
        }
    }

    if total_diagnostics > 0 {
        eprintln!();
        eprintln!("{} offense(s) detected", total_diagnostics);
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}

fn check_file(path: &str, source: &[u8]) -> usize {
    let diagnostics = reukocyte_checker::check(source);

    // Print diagnostics
    for d in &diagnostics {
        println!("{}:{}:{}: {}: {}", path, d.line, d.column, d.rule, d.message);
    }

    diagnostics.len()
}
