//! Reukocyte Checker
//!
//! The main checker that coordinates parsing and rule execution.
//! Inspired by Ruff's architecture - single AST traversal for all rules.

mod checker;
mod diagnostic;
mod locator;
pub mod rules;

pub use locator::LineIndex;

pub use checker::Checker;
pub use diagnostic::Diagnostic;

use ruby_prism::Visit;

/// Check a Ruby source file for violations.
///
/// This is the main entry point that:
/// 1. Parses the source once
/// 2. Runs line-based rules (Layout)
/// 3. Traverses the AST once for all node-based rules (Lint)
pub fn check(source: &[u8]) -> Vec<Diagnostic> {
    let parse_result = ruby_prism::parse(source);
    let mut checker = Checker::new(source);

    // Run line-based rules (no AST needed)
    rules::layout::trailing_whitespace::check(&mut checker);

    // Run AST-based rules (single traversal)
    checker.visit(&parse_result.node());

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
        assert_eq!(diagnostics[0].rule, "Layout/TrailingWhitespace");
    }

    #[test]
    fn test_check_debugger() {
        let source = b"def foo\n  binding.pry\nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "Lint/Debugger");
    }

    #[test]
    fn test_check_multiple_violations() {
        let source = b"def foo  \n  binding.pry\nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 2);
        // Should be sorted by line/column
        assert_eq!(diagnostics[0].rule, "Layout/TrailingWhitespace");
        assert_eq!(diagnostics[1].rule, "Lint/Debugger");
    }
}
