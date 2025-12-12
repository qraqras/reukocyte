mod args;
mod files;
mod output;

use args::Args;
use args::OutputFormat;
use clap::Parser;
use files::collect_ruby_files;
use output::JsonOutput;
use reukocyte_checker::Category;
use reukocyte_checker::Config;
use reukocyte_checker::Diagnostic;
use reukocyte_checker::Severity;
use reukocyte_checker::apply_fixes_filtered;
use reukocyte_checker::check_with_config_and_path;
use reukocyte_checker::load_rubocop_yaml;
use rustc_hash::FxHashMap;
use std::io::Read;
use std::process::ExitCode;

/// Exit codes compatible with RuboCop
#[allow(dead_code)]
mod exit_code {
    pub const SUCCESS: u8 = 0;
    pub const OFFENSES: u8 = 1;
    pub const ERROR: u8 = 2;
}

/// Main entry point
fn main() -> ExitCode {
    let args = Args::parse();
    // Debug output if --debug is enabled
    if args.debug {
        eprintln!("Debug: {:?}", args);
    }
    // Start timing if --display-time is enabled
    let start_time = std::time::Instant::now();
    // Handle stdin mode
    if let Some(ref stdin_file) = args.stdin {
        return handle_stdin(&args, stdin_file);
    }
    // Process files
    let result = run(&args);
    // Display elapsed time if requested
    if args.display_time {
        let elapsed = start_time.elapsed();
        eprintln!("Finished in {:.2} seconds", elapsed.as_secs_f64());
    }
    result
}

/// Load configuration from file or use defaults.
fn load_config(args: &Args) -> Config {
    if let Some(ref config_path) = args.config {
        match load_rubocop_yaml(config_path) {
            Ok(yaml) => Config::from_rubocop_yaml(&yaml),
            Err(e) => {
                eprintln!("Warning: Failed to load config {}: {}", config_path.display(), e);
                Config::default()
            }
        }
    } else {
        // Try to find .rubocop.yml in current directory
        let default_path = std::path::Path::new(".rubocop.yml");
        if args.debug {
            eprintln!("Checking for .rubocop.yml at: {:?}, exists: {}", default_path, default_path.exists());
        }
        if default_path.exists() {
            match load_rubocop_yaml(default_path) {
                Ok(yaml) => {
                    if args.debug {
                        eprintln!("Loaded config from: .rubocop.yml");
                    }
                    Config::from_rubocop_yaml(&yaml)
                }
                Err(e) => {
                    if args.debug {
                        eprintln!("Failed to load .rubocop.yml: {}", e);
                    }
                    Config::default()
                }
            }
        } else {
            Config::default()
        }
    }
}

/// Handle reading from stdin
fn handle_stdin(args: &Args, filename: &std::path::Path) -> ExitCode {
    let mut source = Vec::new();
    if std::io::stdin().read_to_end(&mut source).is_err() {
        eprintln!("Error reading from stdin");
        return ExitCode::from(exit_code::OFFENSES);
    }

    let config = load_config(args);
    let path_str = filename.to_string_lossy();
    let (remaining, _fixed_count) = check_file(&path_str, &source, args, &config);

    if remaining.is_empty() {
        ExitCode::from(exit_code::SUCCESS)
    } else {
        ExitCode::from(exit_code::OFFENSES)
    }
}

/// Run the checker on the given files and return appropriate exit code.
fn run(args: &Args) -> ExitCode {
    // Load configuration
    let config = load_config(args);

    // Collect all Ruby files from the given paths, respecting AllCops.Exclude
    let files = collect_ruby_files(&args.files, &config.all_cops.exclude);

    if files.is_empty() {
        if args.debug {
            eprintln!("No Ruby files found");
        }
        return ExitCode::from(exit_code::SUCCESS);
    }

    let mut total_remaining = 0;
    let mut total_fixed = 0;
    let mut file_results: FxHashMap<String, Vec<Diagnostic>> = FxHashMap::default();
    let mut corrected_counts: FxHashMap<String, usize> = FxHashMap::default();

    for path in &files {
        match std::fs::read(path) {
            Ok(source) => {
                let path_str = path.to_string_lossy().to_string();
                let (remaining, fixed_count) = check_file(&path_str, &source, args, &config);

                total_remaining += remaining.len();
                total_fixed += fixed_count;

                // Always include files in results (RuboCop-compatible)
                file_results.insert(path_str.clone(), remaining);
                if fixed_count > 0 {
                    corrected_counts.insert(path_str, fixed_count);
                }

                // Handle fail-fast
                if args.fail_fast && total_remaining > 0 {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading {}: {}", path.display(), e);
            }
        }
    }

    // Output based on format
    let format = args.output_format();
    match format {
        OutputFormat::Json => {
            let json_output = JsonOutput::new(file_results, corrected_counts);
            println!("{}", json_output.to_json());
        }
        _ => {
            // Print summary for non-JSON formats
            print_summary(args, files.len(), total_remaining, total_fixed);
        }
    }

    if total_remaining > 0 {
        ExitCode::from(exit_code::OFFENSES)
    } else {
        ExitCode::from(exit_code::SUCCESS)
    }
}

/// Check if a diagnostic should be included based on CLI options.
fn should_include_diagnostic(diagnostic: &Diagnostic, args: &Args) -> bool {
    // Filter by --lint (only Lint cops)
    if args.lint && diagnostic.rule_id.category() != Category::Lint {
        return false;
    }
    // Filter by --fix-layout / -x (only Layout cops)
    if args.fix_layout && diagnostic.rule_id.category() != Category::Layout {
        return false;
    }
    // Filter by --only (include only specified cops)
    if let Some(ref only) = args.only {
        let rule_name = diagnostic.rule_id.to_string();
        let short_name = diagnostic.rule_id.name();
        if !only.iter().any(|o| rule_name.eq_ignore_ascii_case(o) || short_name.eq_ignore_ascii_case(o)) {
            return false;
        }
    }
    // Filter by --except (exclude specified cops)
    if let Some(ref except) = args.except {
        let rule_name = diagnostic.rule_id.to_string();
        let short_name = diagnostic.rule_id.name();
        if except.iter().any(|e| rule_name.eq_ignore_ascii_case(e) || short_name.eq_ignore_ascii_case(e)) {
            return false;
        }
    }
    true
}

/// Filter diagnostics based on CLI options.
fn filter_diagnostics(diagnostics: Vec<Diagnostic>, args: &Args) -> Vec<Diagnostic> {
    diagnostics.into_iter().filter(|d| should_include_diagnostic(d, args)).collect()
}

/// Check a file and return (remaining_diagnostics, fixed_count).
fn check_file(path: &str, source: &[u8], args: &Args, config: &Config) -> (Vec<Diagnostic>, usize) {
    let diagnostics = check_with_config_and_path(source, config, Some(std::path::Path::new(path)));
    let diagnostics = filter_diagnostics(diagnostics, args);

    if args.should_fix() && !diagnostics.is_empty() {
        // Create a filter closure that captures the args
        let filter = |diagnostic: &Diagnostic| should_include_diagnostic(diagnostic, args);
        // Apply fixes with filtering
        let (fixed_source, fix_count) = apply_fixes_filtered(Some(path), source, &diagnostics, args.unsafe_fixes(), filter);

        if fix_count > 0 {
            // Write the fixed source back to the file
            if let Err(e) = std::fs::write(path, &fixed_source) {
                eprintln!("Error writing {}: {}", path, e);
            }
        }

        // Get remaining diagnostics (also filtered)
        let remaining = check_with_config_and_path(&fixed_source, config, Some(std::path::Path::new(path)));
        let remaining = filter_diagnostics(remaining, args);
        print_diagnostics(path, &remaining, args);
        (remaining, fix_count)
    } else {
        print_diagnostics(path, &diagnostics, args);
        (diagnostics, 0)
    }
}

/// Print diagnostics based on the selected output format.
fn print_diagnostics(path: &str, diagnostics: &[Diagnostic], args: &Args) {
    match args.output_format() {
        OutputFormat::Json => {
            // JSON output is handled by run() using JsonOutput for RuboCop compatibility
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
                    println!("{}:{}:{}: {}: {}", path, d.line_start, d.column_start, d.severity.code(), d.message);
                }
            }
        }
        OutputFormat::Emacs => {
            for d in diagnostics {
                println!("{}:{}:{}: {}: {}", path, d.line_start, d.column_start, d.severity.code(), d.message);
            }
        }
        OutputFormat::Github => {
            for d in diagnostics {
                let level = match d.severity {
                    Severity::Error | Severity::Fatal => "error",
                    Severity::Warning => "warning",
                    _ => "notice",
                };
                println!("::{} file={},line={},col={}::{}", level, path, d.line_start, d.column_start, d.message);
            }
        }
        OutputFormat::Clang => {
            for d in diagnostics {
                println!("{}:{}:{}: {}: {}", path, d.line_start, d.column_start, d.severity.as_str(), d.message);
            }
        }
        OutputFormat::Files => {
            if !diagnostics.is_empty() {
                println!("{}", path);
            }
        }
    }
}

/// Print summary of the run based on output format.
fn print_summary(args: &Args, file_count: usize, total_diagnostics: usize, total_fixed: usize) {
    match args.output_format() {
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
                eprintln!("{} file(s) inspected, {} offense(s) detected", file_count, total_diagnostics);
            } else {
                eprintln!("{} file(s) inspected, no offenses detected", file_count);
            }
        }
    }
}
