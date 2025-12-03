use std::env;
use std::fs;
use std::process::ExitCode;

use reukocyte_checker::{apply_fixes, check};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    // Parse arguments
    let mut fix_mode = false;
    let mut files: Vec<&str> = Vec::new();

    for arg in &args[1..] {
        match arg.as_str() {
            "--fix" | "-a" => fix_mode = true,
            "--help" | "-h" => {
                print_help();
                return ExitCode::from(0);
            }
            _ if arg.starts_with('-') => {
                eprintln!("Unknown option: {}", arg);
                return ExitCode::from(1);
            }
            _ => files.push(arg),
        }
    }

    if files.is_empty() {
        eprintln!("Usage: reukocyte [--fix] <file>...");
        return ExitCode::from(1);
    }

    let mut total_diagnostics = 0;
    let mut total_fixed = 0;

    for path in files {
        match fs::read(path) {
            Ok(source) => {
                let (diagnostics, fixed) = check_file(path, &source, fix_mode);
                total_diagnostics += diagnostics;
                total_fixed += fixed;
            }
            Err(e) => {
                eprintln!("Error reading {}: {}", path, e);
            }
        }
    }

    if total_fixed > 0 {
        eprintln!();
        eprintln!("{} offense(s) corrected", total_fixed);
    }

    if total_diagnostics > 0 {
        eprintln!();
        eprintln!("{} offense(s) detected", total_diagnostics);
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}

fn print_help() {
    println!("reukocyte - A fast Ruby linter");
    println!();
    println!("USAGE:");
    println!("    reukocyte [OPTIONS] <file>...");
    println!();
    println!("OPTIONS:");
    println!("    --fix, -a    Auto-correct offenses");
    println!("    --help, -h   Print this help message");
}

fn check_file(path: &str, source: &[u8], fix_mode: bool) -> (usize, usize) {
    let diagnostics = check(source);

    if fix_mode && !diagnostics.is_empty() {
        // RuboCop-style: apply_fixes now handles iterative correction internally
        let (fixed_source, fix_count) = apply_fixes(source, &diagnostics, false);

        if fix_count > 0 {
            // Write the fixed source back to the file
            if let Err(e) = fs::write(path, &fixed_source) {
                eprintln!("Error writing {}: {}", path, e);
            }
        }

        // Get remaining diagnostics (unfixable ones)
        let remaining = check(&fixed_source);

        // Print remaining diagnostics
        for d in &remaining {
            println!(
                "{}:{}:{}: {}: {}",
                path,
                d.line_start,
                d.column_start,
                d.rule(),
                d.message
            );
        }

        (remaining.len(), fix_count)
    } else {
        // Print diagnostics
        for d in &diagnostics {
            println!(
                "{}:{}:{}: {}: {}",
                path,
                d.line_start,
                d.column_start,
                d.rule(),
                d.message
            );
        }

        (diagnostics.len(), 0)
    }
}
