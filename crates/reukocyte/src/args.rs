use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Rueko: An extremely fast Ruby linter (Reukocyte)
#[derive(Debug, Parser)]
#[command(name = "rueko")]
#[command(author, version, about, long_about = None)]
#[command(after_help = "For more information, see: https://github.com/qraqras/reukocyte")]
pub struct Args {
    /// List of files or directories to check
    #[arg(value_name = "FILE", default_value = ".")]
    pub files: Vec<PathBuf>,

    // **************** Autocorrection Options ****************
    /// Autocorrect offenses (only when it's safe)
    #[arg(short = 'a', long = "autocorrect")]
    pub autocorrect: bool,

    /// Autocorrect offenses (safe and unsafe)
    #[arg(short = 'A', long = "autocorrect-all")]
    pub autocorrect_all: bool,

    // **************** Rule Selection Options ****************
    /// Run only the given rule(s)
    #[arg(long, value_name = "RULE1,RULE2,...", value_delimiter = ',')]
    pub only: Option<Vec<String>>,

    /// Exclude the given rule(s)
    #[arg(long, value_name = "RULE1,RULE2,...", value_delimiter = ',')]
    pub except: Option<Vec<String>>,

    /// Run only lint rules
    #[arg(short = 'l', long = "lint")]
    pub lint: bool,

    /// Run only layout rules, with autocorrect on
    #[arg(short = 'x', long = "fix-layout")]
    pub fix_layout: bool,

    /// Run only safe rules
    #[arg(long)]
    pub safe: bool,

    // **************** Output Options ****************
    /// Choose an output formatter
    #[arg(short = 'f', long = "format", value_name = "FORMATTER")]
    pub format: Option<OutputFormat>,

    /// Display rule names in offense messages (default: true)
    #[arg(short = 'D', long = "display-cop-names", default_value = "true")]
    pub display_cop_names: bool,

    /// Do not display rule names in offense messages
    #[arg(long = "no-display-cop-names")]
    pub no_display_cop_names: bool,

    /// Write output to a file instead of STDOUT
    #[arg(short = 'o', long = "out", value_name = "FILE")]
    pub output_file: Option<PathBuf>,

    /// Write all output to stderr
    #[arg(long)]
    pub stderr: bool,

    /// Force color output on or off
    #[arg(long)]
    pub color: bool,

    /// Disable color output
    #[arg(long = "no-color")]
    pub no_color: bool,

    // **************** Configuration Options ****************
    /// Specify configuration file
    #[arg(short = 'c', long = "config", value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Pipe source from STDIN, using FILE in offense reports
    #[arg(short = 's', long = "stdin", value_name = "FILE")]
    pub stdin: Option<PathBuf>,

    // **************** Behavior Options ****************
    /// Minimum severity for exit with error code
    #[arg(long = "fail-level", value_name = "SEVERITY")]
    pub fail_level: Option<Severity>,

    /// Inspect files in order and stop after the first file with offenses
    #[arg(short = 'F', long = "fail-fast")]
    pub fail_fast: bool,

    /// Force exclusion of files specified in config
    #[arg(long)]
    pub force_exclusion: bool,

    /// Use available CPUs to execute inspection in parallel
    #[arg(short = 'P', long = "parallel")]
    pub parallel: bool,

    /// Disable parallel execution
    #[arg(long = "no-parallel")]
    pub no_parallel: bool,

    // **************** Debug/Info Options ****************
    /// Display debug info
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,

    /// Display elapsed time in seconds
    #[arg(long = "display-time")]
    pub display_time: bool,
}
impl Args {
    /// Check if any autocorrect mode is enabled
    pub fn should_fix(&self) -> bool {
        // `-a` or `-A` or `-x` enables fixing
        self.autocorrect || self.autocorrect_all || self.fix_layout
    }
    /// Check if unsafe fixes should be applied
    pub fn unsafe_fixes(&self) -> bool {
        self.autocorrect_all
    }
    /// Get the effective output format
    pub fn output_format(&self) -> OutputFormat {
        self.format.unwrap_or_default()
    }
    /// Check if color should be enabled
    pub fn use_color(&self) -> bool {
        if self.no_color {
            false
        } else if self.color {
            true
        } else {
            // Auto-detect based on terminal
            atty_check()
        }
    }
    /// Check if cop names should be displayed
    pub fn show_cop_names(&self) -> bool {
        !self.no_display_cop_names && self.display_cop_names
    }
}

/// Output formatter type
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// JSON format (RuboCop compatible)
    #[value(name = "json", alias = "j")]
    Json,
    /// Simple text output
    #[value(name = "simple", alias = "s")]
    Simple,
    /// Minimal output
    #[value(name = "quiet", alias = "q")]
    Quiet,
    /// Progress display (RuboCop default)
    #[value(name = "progress", alias = "p")]
    Progress,
    /// Clang-style output
    #[value(name = "clang", alias = "c")]
    Clang,
    /// Emacs format
    #[value(name = "emacs", alias = "e")]
    Emacs,
    /// GitHub Actions format
    #[value(name = "github", alias = "g")]
    Github,
    /// Files only
    #[value(name = "files", alias = "fi")]
    Files,
}
impl Default for OutputFormat {
    fn default() -> Self {
        Self::Progress
    }
}

/// Severity level for --fail-level option
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Severity {
    /// Info level
    #[value(name = "info", alias = "I")]
    Info,
    /// Refactor level
    #[value(name = "refactor", alias = "R")]
    Refactor,
    /// Convention level
    #[value(name = "convention", alias = "C")]
    Convention,
    /// Warning level
    #[value(name = "warning", alias = "W")]
    Warning,
    /// Error level
    #[value(name = "error", alias = "E")]
    Error,
    /// Fatal level
    #[value(name = "fatal", alias = "F")]
    Fatal,
}
impl Default for Severity {
    fn default() -> Self {
        Self::Convention
    }
}

/// Check if stdout is a tty (for auto color detection)
fn atty_check() -> bool {
    // Simple check - in real implementation, use atty crate or std::io::IsTerminal
    std::env::var("TERM").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        // Verify that the CLI is valid
        Args::command().debug_assert();
    }

    #[test]
    fn test_default_values() {
        let args = Args::parse_from(["rueko", "."]);
        assert!(!args.autocorrect);
        assert!(!args.autocorrect_all);
        assert!(args.only.is_none());
        assert!(args.except.is_none());
    }

    #[test]
    fn test_autocorrect() {
        let args = Args::parse_from(["rueko", "-a", "."]);
        assert!(args.autocorrect);
        assert!(args.should_fix());
        assert!(!args.unsafe_fixes());
    }

    #[test]
    fn test_autocorrect_all() {
        let args = Args::parse_from(["rueko", "-A", "."]);
        assert!(args.autocorrect_all);
        assert!(args.should_fix());
        assert!(args.unsafe_fixes());
    }

    #[test]
    fn test_only_option() {
        let args = Args::parse_from(["rueko", "--only", "Layout/TrailingWhitespace,Lint/Debugger", "."]);
        assert_eq!(args.only, Some(vec!["Layout/TrailingWhitespace".to_string(), "Lint/Debugger".to_string()]));
    }

    #[test]
    fn test_format_json() {
        let args = Args::parse_from(["rueko", "-f", "json", "."]);
        assert_eq!(args.format, Some(OutputFormat::Json));
    }

    #[test]
    fn test_format_short() {
        let args = Args::parse_from(["rueko", "-f", "j", "."]);
        assert_eq!(args.format, Some(OutputFormat::Json));
    }

    #[test]
    fn test_fail_level() {
        let args = Args::parse_from(["rueko", "--fail-level", "warning", "."]);
        assert_eq!(args.fail_level, Some(Severity::Warning));
    }

    #[test]
    fn test_config_file() {
        let args = Args::parse_from(["rueko", "-c", ".rubocop.yml", "."]);
        assert_eq!(args.config, Some(PathBuf::from(".rubocop.yml")));
    }

    #[test]
    fn test_fix_layout() {
        let args = Args::parse_from(["rueko", "-x", "."]);
        assert!(args.fix_layout);
        assert!(args.should_fix());
    }
}
