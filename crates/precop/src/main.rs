use clap::Parser;
use precop_core::{CheckContext, Cop};
use precop_layout::{TrailingWhitespace, TrailingEmptyLines};
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "precop")]
#[command(about = "A fast Ruby Layout formatter written in Rust")]
#[command(version)]
struct Cli {
    /// Files or directories to check
    #[arg(default_value = ".")]
    paths: Vec<String>,

    /// Check mode (don't modify files, exit with error if issues found)
    #[arg(long, short = 'c')]
    check: bool,

    /// Auto-fix layout issues
    #[arg(long, short = 'f')]
    fix: bool,

    /// Show timing information
    #[arg(long, short = 't')]
    time: bool,

    /// Quiet mode (only show summary)
    #[arg(long, short = 'q')]
    quiet: bool,
}

fn collect_ruby_files(paths: &[String]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for path in paths {
        let path = PathBuf::from(path);
        if path.is_file() {
            if path.extension().is_some_and(|ext| ext == "rb") {
                files.push(path);
            }
        } else if path.is_dir() {
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_file() && entry_path.extension().is_some_and(|ext| ext == "rb") {
                        files.push(entry_path);
                    }
                }
            }
        }
    }

    files
}

fn main() {
    let cli = Cli::parse();
    let start = Instant::now();

    // Collect all Ruby files
    let files = collect_ruby_files(&cli.paths);

    if files.is_empty() {
        if !cli.quiet {
            eprintln!("No Ruby files found");
        }
        std::process::exit(0);
    }

    // Create cops
    let cops: Vec<Box<dyn Cop>> = vec![
        Box::new(TrailingWhitespace::new()),
        Box::new(TrailingEmptyLines::default()),
    ];

    // Process files in parallel
    let results: Vec<_> = files
        .par_iter()
        .map(|file| {
            let source = match fs::read_to_string(file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading {}: {}", file.display(), e);
                    return (file.clone(), vec![]);
                }
            };

            let file_path = file.to_string_lossy().to_string();
            let context = CheckContext {
                source: &source,
                file_path: &file_path,
            };

            let mut offenses = Vec::new();
            for cop in &cops {
                offenses.extend(cop.check(&context));
            }

            (file.clone(), offenses)
        })
        .collect();

    // Report results
    let mut total_offenses = 0;
    let mut files_with_offenses = 0;

    for (file, offenses) in &results {
        if !offenses.is_empty() {
            files_with_offenses += 1;
            total_offenses += offenses.len();

            if !cli.quiet {
                for offense in offenses {
                    println!(
                        "{}:{}:{}: {} {}",
                        file.display(),
                        offense.location.line,
                        offense.location.column,
                        offense.cop_name,
                        offense.message
                    );
                }
            }
        }
    }

    let elapsed = start.elapsed();

    if cli.time || !cli.quiet {
        eprintln!(
            "\n{} files inspected, {} offenses detected in {:.2}ms",
            files.len(),
            total_offenses,
            elapsed.as_secs_f64() * 1000.0
        );
    }

    if total_offenses > 0 && cli.check {
        std::process::exit(1);
    }
}
