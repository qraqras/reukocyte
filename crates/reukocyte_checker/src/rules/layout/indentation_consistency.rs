use crate::checker::Checker;
use crate::rule::Check;
use crate::rule::LayoutRule;
use crate::rule::Rule;
use crate::rule::RuleId;
use reukocyte_macros::check;
use ruby_prism::*;

/// Get the config for this rule
#[inline]
fn config<'a>(checker: &'a Checker<'_>) -> &'a crate::config::layout::indentation_consistency::IndentationConsistency {
    &checker.config().layout.indentation_consistency
}

/// Layout/IndentationConsistency rule.
pub struct IndentationConsistency;
impl Rule for IndentationConsistency {
    const ID: RuleId = RuleId::Layout(LayoutRule::IndentationConsistency);
}
#[check(StatementsNode)]
impl Check<StatementsNode<'_>> for IndentationConsistency {
    fn check(node: &StatementsNode, checker: &mut Checker) {
        // Get the expected indentation based on ancestors
        let expected_indent = calculate_expected_indent(&node.as_node(), checker);
        // Check each statement's indentation
        let body = node.body();
        for statement in body.iter() {
            check_statement_indent(&statement, expected_indent, checker);
        }
    }
}

fn calculate_expected_indent(node: &Node, checker: &Checker) -> usize {
    // Simple implementation: count nesting level
    let ancestors = checker.semantic.ancestors(node as *const Node);
    let mut indent = 0;
    for ancestor_id in ancestors {
        if let Some(ancestor_node) = checker.semantic.node_map.get(&ancestor_id) {
            match **ancestor_node {
                Node::ClassNode { .. } | Node::ModuleNode { .. } | Node::DefNode { .. } | Node::BlockNode { .. } => {
                    indent += 2; // Assume 2 spaces per level
                }
                _ => {}
            }
        }
    }
    indent
}

fn check_statement_indent(statement: &Node, expected_indent: usize, checker: &mut Checker) {
    // Get the line index and check indentation
    let location = statement.location();
    let start_line = checker.line_index().line_index(location.start_offset());
    let line = &checker.line_index().lines()[start_line];
    let line_bytes = line.text;
    let actual_indent = line_bytes.iter().take_while(|&&b| b == b' ').count();
    if actual_indent != expected_indent {
        // Report diagnostic
        checker.report(
            RuleId::Layout(LayoutRule::IndentationConsistency),
            format!("Incorrect indentation. Expected {}, found {}.", expected_indent, actual_indent),
            crate::diagnostic::Severity::Warning,
            location.start_offset(),
            location.end_offset(),
            None,
        );
    }
}
