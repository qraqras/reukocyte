mod args;

use std::fs;
use std::io::Read;
use std::process::ExitCode;

use args::{Args, OutputFormat};
use clap::Parser;
use reukocyte_checker::{Diagnostic, Severity, apply_fixes, check};

/// Exit codes compatible with RuboCop
mod exit_code {
    pub const SUCCESS: u8 = 0;
    pub const OFFENSES: u8 = 1;
    #[allow(dead_code)]
    pub const ERROR: u8 = 2;
}

fn main() -> ExitCode {
    let args = Args::parse();

    if args.debug {
        eprintln!("Debug: {:?}", args);
    }

    let start_time = std::time::Instant::now();

    // Handle stdin mode
    if let Some(ref stdin_file) = args.stdin {
        return handle_stdin(&args, stdin_file);
    }

    // Process files
    let result = run(&args);

    if args.display_time {
        let elapsed = start_time.elapsed();
        eprintln!("Finished in {:.2} seconds", elapsed.as_secs_f64());
    }

    result
}

fn handle_stdin(args: &Args, filename: &std::path::Path) -> ExitCode {
    let mut source = Vec::new();
    if std::io::stdin().read_to_end(&mut source).is_err() {
        eprintln!("Error reading from stdin");
        return ExitCode::from(exit_code::OFFENSES);
    }

    let path_str = filename.to_string_lossy();
    let (diagnostic_count, _) = check_file(&path_str, &source, args);

    if diagnostic_count > 0 {
        ExitCode::from(exit_code::OFFENSES)
    } else {
        ExitCode::from(exit_code::SUCCESS)
    }
}

fn run(args: &Args) -> ExitCode {
    let mut total_diagnostics = 0;
    let mut total_fixed = 0;
    let mut file_count = 0;

    for path in &args.files {
        if path.is_dir() {
            // TODO: Walk directory and find .rb files
            if args.debug {
                eprintln!(
                    "Warning: Directory traversal not yet implemented, skipping: {}",
                    path.display()
                );
            }
            continue;
        }

        match fs::read(path) {
            Ok(source) => {
                file_count += 1;
                let path_str = path.to_string_lossy();
                let (diagnostics, fixed) = check_file(&path_str, &source, args);
                total_diagnostics += diagnostics;
                total_fixed += fixed;

                // Handle fail-fast
                if args.fail_fast && diagnostics > 0 {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading {}: {}", path.display(), e);
            }
        }
    }

    // Print summary based on format
    print_summary(args, file_count, total_diagnostics, total_fixed);

    if total_diagnostics > 0 {
        ExitCode::from(exit_code::OFFENSES)
    } else {
        ExitCode::from(exit_code::SUCCESS)
    }
}

fn check_file(path: &str, source: &[u8], args: &Args) -> (usize, usize) {
    let diagnostics = check(source);

    // TODO: Filter diagnostics based on --only, --except, --lint, --safe options

    if args.should_fix() && !diagnostics.is_empty() {
        let (fixed_source, fix_count) =
            apply_fixes(Some(path), source, &diagnostics, args.unsafe_fixes());

        if fix_count > 0 {
            // Write the fixed source back to the file
            if let Err(e) = fs::write(path, &fixed_source) {
                eprintln!("Error writing {}: {}", path, e);
            }
        }

        // Get remaining diagnostics
        let remaining = check(&fixed_source);
        print_diagnostics(path, &remaining, args);

        (remaining.len(), fix_count)
    } else {
        print_diagnostics(path, &diagnostics, args);
        (diagnostics.len(), 0)
    }
}

fn print_diagnostics(path: &str, diagnostics: &[Diagnostic], args: &Args) {
    let format = args.output_format();

    match format {
        OutputFormat::Json => {
            // TODO: Implement proper JSON array output
            for d in diagnostics {
                println!(
                    r#"{{"path":"{}","line":{},"column":{},"cop":"{}","message":"{}","severity":"{}"}}"#,
                    path,
                    d.line_start,
                    d.column_start,
                    d.rule(),
                    d.message.replace('"', "\\\""),
                    d.severity.as_str()
                );
            }
        }
        OutputFormat::Quiet => {
            // Quiet mode: no output
        }
        OutputFormat::Simple | OutputFormat::Progress => {
            for d in diagnostics {
                if args.show_cop_names() {
                    println!(
                        "{}:{}:{}: {}: {} {}",
                        path,
                        d.line_start,
                        d.column_start,
                        d.severity.code(),
                        d.rule(),
                        d.message
                    );
                } else {
                    println!(
                        "{}:{}:{}: {}: {}",
                        path,
                        d.line_start,
                        d.column_start,
                        d.severity.code(),
                        d.message
                    );
                }
            }
        }
        OutputFormat::Emacs => {
            for d in diagnostics {
                println!(
                    "{}:{}:{}: {}: {}",
                    path,
                    d.line_start,
                    d.column_start,
                    d.severity.code(),
                    d.message
                );
            }
        }
        OutputFormat::Github => {
            for d in diagnostics {
                let level = match d.severity {
                    Severity::Error | Severity::Fatal => "error",
                    Severity::Warning => "warning",
                    _ => "notice",
                };
                println!(
                    "::{} file={},line={},col={}::{}",
                    level, path, d.line_start, d.column_start, d.message
                );
            }
        }
        OutputFormat::Clang => {
            for d in diagnostics {
                println!(
                    "{}:{}:{}: {}: {}",
                    path,
                    d.line_start,
                    d.column_start,
                    d.severity.as_str(),
                    d.message
                );
            }
        }
        OutputFormat::Files => {
            if !diagnostics.is_empty() {
                println!("{}", path);
            }
        }
    }
}

fn print_summary(args: &Args, file_count: usize, total_diagnostics: usize, total_fixed: usize) {
    let format = args.output_format();

    match format {
        OutputFormat::Quiet | OutputFormat::Json | OutputFormat::Files => {
            // No summary for these formats
        }
        _ => {
            if total_fixed > 0 {
                eprintln!();
                eprintln!("{} offense(s) corrected", total_fixed);
            }

            eprintln!();
            if total_diagnostics > 0 {
                eprintln!(
                    "{} file(s) inspected, {} offense(s) detected",
                    file_count, total_diagnostics
                );
            } else {
                eprintln!("{} file(s) inspected, no offenses detected", file_count);
            }
        }
    }
}
