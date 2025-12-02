//! Layout/TrailingWhitespace cop
//!
//! Checks for trailing whitespace at the end of lines.
//!
//! # Examples
//!
//! ```ruby
//! # bad
//! def foo
//!   bar
//! end
//!
//! # good
//! def foo
//!   bar
//! end
//! ```

use precop_core::cop::{CheckContext, Cop};
use precop_core::offense::{Location, Offense};

/// Checks for trailing whitespace at the end of lines.
#[derive(Debug, Default)]
pub struct TrailingWhitespace {
    /// Whether to allow trailing whitespace in heredocs
    pub allow_in_heredoc: bool,
}

impl TrailingWhitespace {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_allow_in_heredoc(mut self, allow: bool) -> Self {
        self.allow_in_heredoc = allow;
        self
    }
}

impl Cop for TrailingWhitespace {
    fn name(&self) -> &'static str {
        "Layout/TrailingWhitespace"
    }

    fn check(&self, context: &CheckContext) -> Vec<Offense> {
        let mut offenses = Vec::new();

        for (line_num, line) in context.source.lines().enumerate() {
            let trimmed_len = line.trim_end().len();
            let original_len = line.len();

            if trimmed_len < original_len {
                let whitespace_len = original_len - trimmed_len;

                offenses.push(Offense::new(
                    self.name(),
                    "Trailing whitespace detected.",
                    context.file_path,
                    Location::new(line_num + 1, trimmed_len + 1, whitespace_len),
                ));
            }
        }

        offenses
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn autocorrect(&self, source: &str) -> String {
        source
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_trailing_whitespace() {
        let cop = TrailingWhitespace::new();
        let context = CheckContext {
            source: "def foo\n  bar\nend\n",
            file_path: "test.rb",
        };

        let offenses = cop.check(&context);
        assert!(offenses.is_empty());
    }

    #[test]
    fn test_trailing_whitespace_detected() {
        let cop = TrailingWhitespace::new();
        let context = CheckContext {
            source: "def foo  \n  bar\nend\n",
            file_path: "test.rb",
        };

        let offenses = cop.check(&context);
        assert_eq!(offenses.len(), 1);
        assert_eq!(offenses[0].location.line, 1);
    }

    #[test]
    fn test_autocorrect() {
        let cop = TrailingWhitespace::new();
        let source = "def foo  \n  bar  \nend\n";
        let corrected = cop.autocorrect(source);

        assert_eq!(corrected, "def foo\n  bar\nend");
    }
}
