//! Expression analysis - runs rules on expression nodes.

use ruby_prism::CallNode;

use crate::Checker;
use crate::Diagnostic;

/// Run rules on a CallNode.
pub fn call(checker: &mut Checker, node: &CallNode) {
    // Lint/Debugger
    let source = checker.source();
    let mut diagnostics = Vec::new();
    reukocyte_lint::rules::debugger::check_node(source, node, |d| {
        diagnostics.push(Diagnostic {
            rule: d.rule,
            message: d.message,
            start: d.start,
            end: d.end,
            line: d.line,
            column: d.column,
        });
    });
    for d in diagnostics {
        checker.add_diagnostic(d);
    }
}
