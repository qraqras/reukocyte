mod checker;
mod config;
mod conflict;
mod corrector;
pub mod custom_nodes;
mod diagnostic;
mod fix;
mod locator;
mod rule;
mod semantic;
mod utility;

pub mod rules;

pub use checker::Checker;
pub use config::{
    load_rubocop_yaml, parse_rubocop_yaml, AllCopsConfig, Config,
    InheritFrom, LayoutConfig, LoadError, RubocopYaml,
};
pub use conflict::ConflictRegistry;
pub use corrector::{ClobberingError, Corrector};
pub use diagnostic::{Applicability, Diagnostic, Edit, Fix, Severity};
pub use fix::{InfiniteCorrectionLoop, apply_fixes, apply_fixes_filtered, apply_fixes_with_loop_detection, apply_fixes_with_remaining};
pub use locator::LineIndex;
pub use rule::{Category, Check, LayoutRule, LintRule, Rule, RuleId};

use ruby_prism::Visit;

/// Check a Ruby source file for violations with default configuration.
///
/// This is the main entry point that:
/// 1. Parses the source once
/// 2. Traverses the AST once for all node-based rules (Lint)
/// 3. Runs line-based rules (Layout) - can use info from AST phase
pub fn check(source: &[u8]) -> Vec<Diagnostic> {
    check_with_config(source, &Config::default())
}

/// Check a Ruby source file for violations with custom configuration.
pub fn check_with_config(source: &[u8], config: &Config) -> Vec<Diagnostic> {
    check_with_config_and_path(source, config, None)
}

/// Check a Ruby source file for violations with custom configuration and file path.
///
/// The file path is used for cop-specific Exclude pattern matching.
pub fn check_with_config_and_path(
    source: &[u8],
    config: &Config,
    file_path: Option<&str>,
) -> Vec<Diagnostic> {
    let parse_result = ruby_prism::parse(source);
    let mut checker = if let Some(path) = file_path {
        Checker::with_file_path(source, config, path)
    } else {
        Checker::new(source, config)
    };

    // Phase 1: Build node index (pre-index all nodes before rules run)
    checker.build_index(&parse_result.node());

    // Phase 2: Run AST-based rules (single traversal)
    checker.visit(&parse_result.node());

    // Phase 3: Run line-based rules (after AST, can use collected info)
    rules::layout::trailing_whitespace::check(&mut checker);
    rules::layout::trailing_empty_lines::check(&mut checker);
    rules::layout::leading_empty_lines::check(&mut checker);
    rules::layout::empty_lines::check(&mut checker);
    rules::layout::indentation_style::check(&mut checker);

    checker.into_diagnostics()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_empty_source() {
        let source = b"";
        let diagnostics = check(source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_check_clean_source() {
        let source = b"def foo\n  bar\nend\n";
        let diagnostics = check(source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_check_trailing_whitespace() {
        let source = b"def foo  \n  bar\nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule(), "Layout/TrailingWhitespace");
    }

    #[test]
    fn test_check_debugger() {
        let source = b"def foo\n  binding.pry\nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule(), "Lint/Debugger");
    }

    #[test]
    fn test_check_multiple_violations() {
        let source = b"def foo  \n  binding.pry\nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 2);
        // Should be sorted by line/column
        assert_eq!(diagnostics[0].rule(), "Layout/TrailingWhitespace");
        assert_eq!(diagnostics[1].rule(), "Lint/Debugger");
    }
}
