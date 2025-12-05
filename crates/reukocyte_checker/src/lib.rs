mod analyze;
mod checker;
mod config;
mod conflict;
mod corrector;
mod diagnostic;
mod fix;
mod locator;
mod rule;
pub mod utils;

pub mod rules;

pub use checker::Checker;
pub use config::{Config, LayoutConfig};
pub use conflict::ConflictRegistry;
pub use corrector::{ClobberingError, Corrector};
pub use diagnostic::{Applicability, Diagnostic, Edit, Fix, Severity};
pub use fix::{InfiniteCorrectionLoop, apply_fixes, apply_fixes_with_loop_detection, apply_fixes_with_remaining};
pub use locator::LineIndex;
pub use rule::{Category, LayoutRule, LintRule, RuleId};

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
    let parse_result = ruby_prism::parse(source);
    let mut checker = Checker::new(source, config);

    // Run AST-based rules (single traversal)
    checker.visit(&parse_result.node());

    // Run line-based rules (after AST, can use collected info)
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
