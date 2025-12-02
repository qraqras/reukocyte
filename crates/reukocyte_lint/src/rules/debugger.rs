//! Lint/Debugger
//!
//! Detects debugger statements left in the code.
//!
//! # Examples
//!
//! ```ruby
//! # bad
//! def foo
//!   binding.pry
//!   do_something
//! end
//!
//! # bad
//! def bar
//!   debugger
//!   do_something
//! end
//!
//! # good
//! def foo
//!   do_something
//! end
//! ```

use ruby_prism::{ParseResult, Visit};

use crate::Diagnostic;

const RULE_NAME: &str = "Lint/Debugger";

/// Standalone debugger method names to detect (no receiver)
const STANDALONE_DEBUGGERS: &[&[u8]] = &[
    b"debugger",
    b"byebug",
    b"remote_byebug",
];

/// Debugger receiver/method combinations
const DEBUGGER_RECEIVERS: &[(&[u8], &[u8])] = &[
    (b"binding", b"pry"),
    (b"binding", b"remote_pry"),
    (b"binding", b"pry_remote"),
    (b"binding", b"irb"),
    (b"binding", b"console"),
    (b"Pry", b"rescue"),
];

/// Check for debugger statements in the source.
pub fn check(source: &[u8], parse_result: &ParseResult<'_>) -> Vec<Diagnostic> {
    let mut visitor = DebuggerVisitor {
        source,
        diagnostics: Vec::new(),
    };

    visitor.visit(&parse_result.node());

    visitor.diagnostics
}

struct DebuggerVisitor<'a> {
    source: &'a [u8],
    diagnostics: Vec<Diagnostic>,
}

impl DebuggerVisitor<'_> {
    fn add_diagnostic(&mut self, message: String, start_offset: usize, end_offset: usize) {
        let (line, column) = self.offset_to_line_column(start_offset);

        self.diagnostics.push(Diagnostic {
            rule: RULE_NAME,
            message,
            start: start_offset,
            end: end_offset,
            line,
            column,
        });
    }

    fn offset_to_line_column(&self, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut column = 1;

        for (i, &byte) in self.source.iter().enumerate() {
            if i >= offset {
                break;
            }
            if byte == b'\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        (line, column)
    }
}

impl Visit<'_> for DebuggerVisitor<'_> {
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode) {
        let method_name = node.name().as_slice();
        let location = node.location();

        // Check for standalone debugger calls (e.g., `debugger`, `byebug`)
        if node.receiver().is_none() {
            for &debugger_method in STANDALONE_DEBUGGERS {
                if method_name == debugger_method {
                    self.add_diagnostic(
                        format!(
                            "Debugger statement `{}` detected.",
                            String::from_utf8_lossy(debugger_method)
                        ),
                        location.start_offset(),
                        location.end_offset(),
                    );
                    // Continue visiting to find more debuggers
                    ruby_prism::visit_call_node(self, node);
                    return;
                }
            }
        }

        // Check for receiver.method calls (e.g., `binding.pry`)
        if let Some(receiver) = node.receiver() {
            let receiver_name: Option<&[u8]> = if let Some(call) = receiver.as_call_node() {
                Some(call.name().as_slice())
            } else if let Some(const_node) = receiver.as_constant_read_node() {
                Some(const_node.name().as_slice())
            } else {
                None
            };

            if let Some(recv_name) = receiver_name {
                for &(expected_recv, expected_method) in DEBUGGER_RECEIVERS {
                    if recv_name == expected_recv && method_name == expected_method {
                        self.add_diagnostic(
                            format!(
                                "Debugger statement `{}.{}` detected.",
                                String::from_utf8_lossy(expected_recv),
                                String::from_utf8_lossy(expected_method)
                            ),
                            location.start_offset(),
                            location.end_offset(),
                        );
                        ruby_prism::visit_call_node(self, node);
                        return;
                    }
                }
            }
        }

        // Continue visiting child nodes
        ruby_prism::visit_call_node(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_source(source: &[u8]) -> Vec<Diagnostic> {
        let parse_result = ruby_prism::parse(source);
        check(source, &parse_result)
    }

    #[test]
    fn test_no_debugger() {
        let source = b"def foo\n  bar\nend\n";
        let diagnostics = check_source(source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_binding_pry() {
        let source = b"def foo\n  binding.pry\nend\n";
        let diagnostics = check_source(source);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "Lint/Debugger");
        assert!(diagnostics[0].message.contains("binding.pry"));
    }

    #[test]
    fn test_byebug() {
        let source = b"def foo\n  byebug\nend\n";
        let diagnostics = check_source(source);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("byebug"));
    }

    #[test]
    fn test_debugger() {
        let source = b"def foo\n  debugger\nend\n";
        let diagnostics = check_source(source);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("debugger"));
    }

    #[test]
    fn test_binding_irb() {
        let source = b"def foo\n  binding.irb\nend\n";
        let diagnostics = check_source(source);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("binding.irb"));
    }

    #[test]
    fn test_multiple_debuggers() {
        let source = b"def foo\n  binding.pry\n  debugger\nend\n";
        let diagnostics = check_source(source);
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn test_pry_rescue() {
        let source = b"Pry.rescue { foo }\n";
        let diagnostics = check_source(source);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("Pry.rescue"));
    }
}
