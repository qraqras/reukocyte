//! Layout/TrailingEmptyLines cop
//!
//! Checks for trailing blank lines and final newlines.
//!
//! # Examples
//!
//! ```ruby
//! # bad - no final newline
//! def foo
//!   bar
//! end
//!
//! # bad - too many trailing newlines
//! def foo
//!   bar
//! end
//!
//!
//! # good - exactly one final newline
//! def foo
//!   bar
//! end
//! ```

use precop_core::cop::{CheckContext, Cop};
use precop_core::offense::{Location, Offense};

/// Checks for trailing blank lines and final newlines.
#[derive(Debug)]
pub struct TrailingEmptyLines {
    /// Whether a final newline is required
    pub final_newline: bool,
}

impl Default for TrailingEmptyLines {
    fn default() -> Self {
        Self {
            final_newline: true,
        }
    }
}

impl TrailingEmptyLines {
    pub fn new(final_newline: bool) -> Self {
        Self { final_newline }
    }
}

impl Cop for TrailingEmptyLines {
    fn name(&self) -> &'static str {
        "Layout/TrailingEmptyLines"
    }

    fn check(&self, context: &CheckContext) -> Vec<Offense> {
        let mut offenses = Vec::new();
        let source = context.source;

        if source.is_empty() {
            return offenses;
        }

        let ends_with_newline = source.ends_with('\n');
        let line_count = source.lines().count();

        if self.final_newline {
            // Check for missing final newline
            if !ends_with_newline {
                offenses.push(Offense::new(
                    self.name(),
                    "Final newline missing.",
                    context.file_path,
                    Location::new(line_count, 1, 1),
                ));
            }

            // Check for multiple trailing newlines
            let trimmed = source.trim_end_matches('\n');
            let trailing_newlines = source.len() - trimmed.len();

            if trailing_newlines > 1 {
                offenses.push(Offense::new(
                    self.name(),
                    format!(
                        "{} trailing blank lines detected.",
                        trailing_newlines - 1
                    ),
                    context.file_path,
                    Location::new(line_count, 1, trailing_newlines - 1),
                ));
            }
        } else {
            // final_newline: false - no trailing newline allowed
            if ends_with_newline {
                offenses.push(Offense::new(
                    self.name(),
                    "Trailing newline detected.",
                    context.file_path,
                    Location::new(line_count, 1, 1),
                ));
            }
        }

        offenses
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn autocorrect(&self, source: &str) -> String {
        let trimmed = source.trim_end_matches('\n');

        if self.final_newline {
            format!("{}\n", trimmed)
        } else {
            trimmed.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_final_newline() {
        let cop = TrailingEmptyLines::default();
        let context = CheckContext {
            source: "def foo\n  bar\nend\n",
            file_path: "test.rb",
        };

        let offenses = cop.check(&context);
        assert!(offenses.is_empty());
    }

    #[test]
    fn test_missing_final_newline() {
        let cop = TrailingEmptyLines::default();
        let context = CheckContext {
            source: "def foo\n  bar\nend",
            file_path: "test.rb",
        };

        let offenses = cop.check(&context);
        assert_eq!(offenses.len(), 1);
        assert!(offenses[0].message.contains("missing"));
    }

    #[test]
    fn test_multiple_trailing_newlines() {
        let cop = TrailingEmptyLines::default();
        let context = CheckContext {
            source: "def foo\n  bar\nend\n\n\n",
            file_path: "test.rb",
        };

        let offenses = cop.check(&context);
        assert_eq!(offenses.len(), 1);
        assert!(offenses[0].message.contains("trailing blank lines"));
    }

    #[test]
    fn test_autocorrect() {
        let cop = TrailingEmptyLines::default();
        let source = "def foo\nend\n\n\n";
        let corrected = cop.autocorrect(source);

        assert_eq!(corrected, "def foo\nend\n");
    }
}
