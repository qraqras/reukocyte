use crate::Checker;
use crate::rule::{Check, LintRule, Rule, RuleId};
use reukocyte_macros::check;
use ruby_prism::CallNode;

/// Standalone debugger method names to detect (no receiver)
const STANDALONE_DEBUGGERS: &[&[u8]] = &[b"debugger", b"byebug", b"remote_byebug"];

/// Debugger receiver/method combinations
const DEBUGGER_RECEIVERS: &[(&[u8], &[u8])] = &[
    (b"binding", b"pry"),
    (b"binding", b"remote_pry"),
    (b"binding", b"pry_remote"),
    (b"binding", b"irb"),
    (b"binding", b"console"),
    (b"Pry", b"rescue"),
];

/// Lint/Debugger rule - detects debugger statements left in code.
///
/// Note: No fix is provided because removing debugger statements
/// may have side effects (e.g., debugging in production).
pub struct Debugger;

impl Rule for Debugger {
    const ID: RuleId = RuleId::Lint(LintRule::Debugger);
}

/// Get the config for this rule
#[inline]
fn config<'a>(checker: &'a Checker<'_>) -> &'a crate::config::lint::debugger::DebuggerConfig {
    &checker.config().lint.debugger
}

#[check(CallNode)]
impl Check<CallNode<'_>> for Debugger {
    fn check(node: &CallNode, checker: &mut Checker) {
        let cfg = config(checker);
        if !cfg.enabled {
            return;
        }
        let severity = cfg.severity;

        let method_name = node.name().as_slice();
        let location = node.location();

        // Check for standalone debugger calls (e.g., `debugger`, `byebug`)
        if node.receiver().is_none() {
            for &debugger_method in STANDALONE_DEBUGGERS {
                if method_name == debugger_method {
                    checker.report(
                        Self::ID,
                        format!("Debugger statement `{}` detected.", String::from_utf8_lossy(debugger_method)),
                        severity,
                        location.start_offset(),
                        location.end_offset(),
                        None,
                    );
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
                        checker.report(
                            Self::ID,
                            format!(
                                "Debugger statement `{}.{}` detected.",
                                String::from_utf8_lossy(expected_recv),
                                String::from_utf8_lossy(expected_method)
                            ),
                            severity,
                            location.start_offset(),
                            location.end_offset(),
                            None,
                        );
                        return;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::check;
    use crate::rule::{LintRule, RuleId};

    #[test]
    fn test_no_debugger() {
        let source = b"def foo\n  bar\nend\n";
        let diagnostics = check(source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_binding_pry() {
        let source = b"def foo\n  binding.pry\nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule(), "Lint/Debugger");
        assert!(diagnostics[0].message.contains("binding.pry"));
    }

    #[test]
    fn test_byebug() {
        let source = b"def foo\n  byebug\nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("byebug"));
    }

    #[test]
    fn test_debugger() {
        let source = b"def foo\n  debugger\nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("debugger"));
    }

    #[test]
    fn test_binding_irb() {
        let source = b"def foo\n  binding.irb\nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("binding.irb"));
    }

    #[test]
    fn test_multiple_debuggers() {
        let source = b"def foo\n  binding.pry\n  debugger\nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn test_pry_rescue() {
        let source = b"Pry.rescue { foo }\n";
        let diagnostics = check(source);
        // Filter only Debugger diagnostics
        let debugger_diagnostics: Vec<_> = diagnostics.iter().filter(|d| d.rule_id == RuleId::Lint(LintRule::Debugger)).collect();
        assert_eq!(debugger_diagnostics.len(), 1);
        assert!(debugger_diagnostics[0].message.contains("Pry.rescue"));
    }
}
