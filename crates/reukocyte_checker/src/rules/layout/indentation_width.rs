use crate::checker::Checker;
use crate::config::layout::def_end_alignment::EnforcedStyleAlignWith as DefAlignWith;
use crate::config::layout::end_alignment::EnforcedStyleAlignWith as EndAlignWith;
use crate::config::layout::indentation_consistency::EnforcedStyle;
use crate::diagnostic::{Edit, Fix, Severity};
use crate::rule::{LayoutRule, RuleId};
use crate::utils::first_part_of_call_chain;
use ruby_prism::*;

/// Rule identifier for Layout/IndentationWidth.
pub const RULE_ID: RuleId = RuleId::Layout(LayoutRule::IndentationWidth);

/// Handle StatementsNode for indentation width checking.
pub fn on_statements(statements: &StatementsNode, checker: &mut Checker) {
    if let Some(parent) = checker.parent() {
        match parent {
            Node::BeginNode { .. } => check_begin_node(&parent.as_begin_node().unwrap(), statements, checker),
            Node::BlockNode { .. } => check_block_node(&parent.as_block_node().unwrap(), statements, checker),
            Node::ClassNode { .. } => check_class_node(&parent.as_class_node().unwrap(), statements, checker),
            Node::DefNode { .. } => check_def_node(&parent.as_def_node().unwrap(), statements, checker),
            Node::ElseNode { .. } => check_indentation(&parent.location(), statements, checker, EnforcedStyle::Normal),
            Node::EnsureNode { .. } => check_indentation(&parent.location(), statements, checker, EnforcedStyle::Normal),
            Node::ForNode { .. } => check_indentation(&parent.location(), statements, checker, EnforcedStyle::Normal),
            Node::IfNode { .. } => check_if_node(&parent.as_if_node().unwrap(), statements, checker, None),
            Node::InNode { .. } => check_indentation(&parent.location(), statements, checker, EnforcedStyle::Normal),
            Node::RescueNode { .. } => check_indentation(&parent.location(), statements, checker, EnforcedStyle::Normal),
            Node::UntilNode { .. } => check_until_node(&parent.as_until_node().unwrap(), statements, checker, None),
            Node::WhenNode { .. } => check_indentation(&parent.location(), statements, checker, EnforcedStyle::Normal),
            Node::WhileNode { .. } => check_while_node(&parent.as_while_node().unwrap(), statements, checker, None),
            _ => {}
        }
    };
}

/// Check BeginNode for indentation width violations.
fn check_begin_node(begin_node: &BeginNode, statements: &StatementsNode, checker: &mut Checker) {
    // Following RuboCop's approach, it generally aligns with the `end` keyword.
    match (begin_node.begin_keyword_loc(), begin_node.end_keyword_loc()) {
        (Some(_begin), Some(end)) => check_indentation(&end, statements, checker, EnforcedStyle::Normal),
        (Some(begin), None) => check_indentation(&begin, statements, checker, EnforcedStyle::Normal),
        (None, Some(end)) => check_indentation(&end, statements, checker, EnforcedStyle::Normal),
        (None, None) => {}
    }
}

// Check BlockNode for indentation width violations.
fn check_block_node(block_node: &BlockNode, statements: &StatementsNode, checker: &mut Checker) {
    let closing_loc = block_node.closing_loc();
    // Skip single-line blocks like `foo.each { |x| bar }`
    if !checker.line_index().is_first_on_line(closing_loc.start_offset()) {
        return;
    }
    match checker.config().layout.indentation_consistency.enforced_style {
        EnforcedStyle::Normal => check_indentation(&closing_loc, statements, checker, EnforcedStyle::Normal),
        EnforcedStyle::IndentedInternalMethods => check_members(&closing_loc, statements, checker),
    }
}

/// Check DefNode for indentation width violations.
/// If `def` is a method call argument (e.g., `private def x`), refer to Layout/DefEndAlignment config.
fn check_def_node(def_node: &DefNode, statements: &StatementsNode, checker: &mut Checker) {
    // Check if DefNode is a method call argument.
    // AST structure: CallNode -> ArgumentsNode -> DefNode -> StatementsNode
    let call_node = checker.ancestor(2).and_then(|ancestor| ancestor.as_call_node());
    match call_node {
        Some(call_node) => match checker.config().layout.def_end_alignment.enforced_style_align_with {
            DefAlignWith::StartOfLine => check_indentation(&call_node.location(), statements, checker, EnforcedStyle::Normal),
            DefAlignWith::Def => check_indentation(&def_node.location(), statements, checker, EnforcedStyle::Normal),
        },
        None => check_indentation(&def_node.location(), statements, checker, EnforcedStyle::Normal),
    }
}

/// Check IfNode for indentation width violations.
fn check_if_node(if_node: &IfNode, statements: &StatementsNode, checker: &mut Checker, base: Option<&Location>) {
    // Skip assignment if: `variable = if condition ... end`
    if checker.is_ignored_node(if_node.location().start_offset(), if_node.location().end_offset()) {
        return;
    }
    // Skip ternary operator: `condition ? then_expr : else_expr`
    if if_node.if_keyword_loc().is_none() {
        return;
    }
    // Skip modifier if: `do_something if condition`
    if if_node.end_keyword_loc().is_none() {
        return;
    }
    match base {
        Some(base) => check_indentation(base, statements, checker, EnforcedStyle::Normal),
        None => check_indentation(&if_node.location(), statements, checker, EnforcedStyle::Normal),
    }
}

/// Check ClassNode for indentation width violations.
fn check_class_node(class_node: &ClassNode, statements: &StatementsNode, checker: &mut Checker) {
    // Skip single-line class: `class Foo; def bar; end; end`
    if checker.line_index().is_first_on_line(statements.location().start_offset()) {
        check_members(&class_node.location(), statements, checker);
    }
}

/// Check UntilNode for indentation width violations.
fn check_until_node(until_node: &UntilNode, statements: &StatementsNode, checker: &mut Checker, base: Option<&Location>) {
    // Skip assignment until: `variable = until condition ... end`
    if checker.is_ignored_node(until_node.location().start_offset(), until_node.location().end_offset()) {
        return;
    }
    // Skip if condition is on its own line (line break before condition)
    if checker.line_index.is_first_on_line(until_node.predicate().location().start_offset()) {
        match base {
            Some(base) => check_indentation(base, statements, checker, EnforcedStyle::Normal),
            None => check_indentation(&until_node.location(), statements, checker, EnforcedStyle::Normal),
        }
    }
}
/// Check WhileNode for indentation width violations.
fn check_while_node(while_node: &WhileNode, statements: &StatementsNode, checker: &mut Checker, base: Option<&Location>) {
    // Skip assignment while: `variable = while condition ... end`
    if checker.is_ignored_node(while_node.location().start_offset(), while_node.location().end_offset()) {
        return;
    }
    // Skip if condition is on its own line (line break before condition)
    if checker.line_index.is_first_on_line(while_node.predicate().location().start_offset()) {
        match base {
            Some(base) => check_indentation(base, statements, checker, EnforcedStyle::Normal),
            None => check_indentation(&while_node.location(), statements, checker, EnforcedStyle::Normal),
        }
    }
}

/// Check assignment nodes for indentation width violations.
pub fn check_assignment(node: Node, rhs: Node, checker: &mut Checker) {
    let rhs = first_part_of_call_chain(rhs);
    if let Some(rhs) = rhs {
        let should_variable_alignment = match checker.config().layout.end_alignment.enforced_style_align_with {
            EndAlignWith::StartOfLine => !checker.line_index().is_first_on_line(rhs.location().start_offset()),
            EndAlignWith::Keyword => false,
            EndAlignWith::Variable => !checker.line_index().is_first_on_line(rhs.location().start_offset()),
        };
        let base = if should_variable_alignment { &node } else { &rhs };
        match rhs {
            Node::IfNode { .. } => {
                let if_node = rhs.as_if_node().unwrap();
                if let Some(statements) = &if_node.statements() {
                    check_if_node(&if_node, statements, checker, Some(&base.location()))
                }
                checker.ignore_node(&if_node.location());
            }
            Node::UntilNode { .. } => {
                let until_node = rhs.as_until_node().unwrap();
                if let Some(statements) = &until_node.statements() {
                    check_until_node(&until_node, statements, checker, Some(&base.location()))
                }
                checker.ignore_node(&until_node.location());
            }
            Node::WhileNode { .. } => {
                let while_node = rhs.as_while_node().unwrap();
                if let Some(statements) = &while_node.statements() {
                    check_while_node(&while_node, statements, checker, Some(&base.location()))
                }
                checker.ignore_node(&while_node.location());
            }
            _ => {}
        }
    }
}

/// ****************************************************************

/// Check indentation of body relative to base.
///
/// NOTE: This rule only handles spaces for indentation, not tabs (same as RuboCop).
pub fn check_indentation(base_loc: &Location, statements: &StatementsNode, checker: &mut Checker, style: EnforcedStyle) {
    // Determine if we should skip the check.
    if !should_check(base_loc, statements, checker) {
        return;
    }

    // Calculate actual indentation relative to base (can be negative).
    let indentation = checker
        .line_index()
        .column_offset_between(base_loc.start_offset(), statements.location().start_offset());
    let configured_indentation_width = checker.config.layout.indentation_width.width;
    let column_delta = configured_indentation_width - indentation;

    // Skip if indentation is correct.
    if column_delta == 0 {
        return;
    }

    // Get the first statement for reporting.
    let Some(first_body) = statements.body().iter().next() else {
        return;
    };
    let report_node_loc = first_body.location();
    let report_start = report_node_loc.start_offset();
    let report_end = report_node_loc.end_offset();

    // Prepare fix.
    let line_start = checker.line_index().line_start_offset(report_start);
    let fix = if 0 <= column_delta {
        let spaces = " ".repeat(column_delta as usize);
        Some(Fix::safe(vec![Edit::insertion(line_start, spaces)]))
    } else {
        let remove_count = (-column_delta) as usize;
        let end = line_start + remove_count.min(report_start - line_start);
        Some(Fix::safe(vec![Edit::deletion(line_start, end)]))
    };

    // Report diagnostic.
    checker.report(
        RULE_ID,
        format!(
            "Use {} (not {}) spaces for {} indentation.",
            configured_indentation_width,
            indentation,
            style.as_str()
        ),
        Severity::Convention,
        report_start,
        report_end,
        fix,
    );
}

fn check_member_indentation(base_loc: &Location, node: &Node, checker: &mut Checker, style: EnforcedStyle) {
    // Determine if we should skip the check.
    if !should_check(base_loc, node, checker) {
        return;
    }

    // Calculate actual indentation relative to base (can be negative).
    let indentation = checker
        .line_index()
        .column_offset_between(base_loc.start_offset(), node.location().start_offset());
    let configured_indentation_width = checker.config.layout.indentation_width.width;
    let column_delta = configured_indentation_width - indentation;

    // Skip if indentation is correct.
    if column_delta == 0 {
        return;
    }

    // Get the node location for reporting.
    let report_node_loc = node.location();
    let report_start = report_node_loc.start_offset();
    let report_end = report_node_loc.end_offset();

    // Prepare fix.
    let line_start = checker.line_index().line_start_offset(report_start);
    let fix = if 0 <= column_delta {
        let spaces = " ".repeat(column_delta as usize);
        Some(Fix::safe(vec![Edit::insertion(line_start, spaces)]))
    } else {
        let remove_count = (-column_delta) as usize;
        let end = line_start + remove_count.min(report_start - line_start);
        Some(Fix::safe(vec![Edit::deletion(line_start, end)]))
    };

    // Report diagnostic.
    checker.report(
        RULE_ID,
        format!(
            "Use {} (not {}) spaces for {} indentation.",
            configured_indentation_width,
            indentation,
            style.as_str()
        ),
        Severity::Convention,
        report_start,
        report_end,
        fix,
    );
}

/// Check class/module body members for indentation width violations.
pub fn check_members(base_loc: &Location, statements: &StatementsNode, checker: &mut Checker) {
    let body = statements.body();
    if let Some(first) = body.iter().next() {
        check_member_indentation(base_loc, &first, checker, EnforcedStyle::Normal);
    }
    if body.iter().count() <= 1 {
        return;
    }
    match checker.config().layout.indentation_consistency.enforced_style {
        EnforcedStyle::Normal => {
            for member in body.iter().skip(1) {
                check_member_indentation(base_loc, &member, checker, EnforcedStyle::Normal);
            }
        }
        EnforcedStyle::IndentedInternalMethods => {
            let mut iter = body.iter();
            if let Some(prev) = iter.next() {
                for curr in iter {
                    check_member_indentation(&prev.location(), &curr, checker, EnforcedStyle::IndentedInternalMethods);
                }
            }
        }
    }
}

/// Determine if we should skip indentation check.
fn should_check(_base_loc: &Location, node: &Node, checker: &Checker) -> bool {
    // TODO: not implemented: allowed_line?
    // return true if allowed_line?(base_loc)

    if !checker.line_index().is_first_on_line(node.location().start_offset()) {
        return false;
    }
    if let Some(statements) = &node.as_statements_node() {
        if starts_with_access_modifier(statements) {
            return false;
        }
    }
    true
}

/// Check if the body starts with an access modifier (private, protected, public).
fn starts_with_access_modifier(statements: &StatementsNode) -> bool {
    if let Some(first) = statements.body().iter().next() {
        if let Some(call_node) = first.as_call_node() {
            if call_node.receiver().is_none() && call_node.arguments().is_none() {
                let method_name = call_node.name().as_slice();
                return method_name == b"private" || method_name == b"protected" || method_name == b"public";
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::check;

    fn check_source(source: &str) -> Vec<crate::Diagnostic> {
        check(source.as_bytes())
    }

    #[test]
    fn test_correct_indentation_in_def() {
        let source = r#"
def foo
  bar
end
"#;
        let diagnostics = check_source(source);
        let indentation_errors: Vec<_> = diagnostics.iter().filter(|d| d.rule_id == super::RULE_ID).collect();
        assert!(indentation_errors.is_empty(), "Expected no indentation errors, got: {:?}", indentation_errors);
    }

    #[test]
    fn test_incorrect_indentation_in_def_too_many_spaces() {
        let source = r#"
def foo
    bar
end
"#;
        let diagnostics = check_source(source);
        let indentation_errors: Vec<_> = diagnostics.iter().filter(|d| d.rule_id == super::RULE_ID).collect();
        assert_eq!(indentation_errors.len(), 1, "Expected 1 indentation error, got: {:?}", indentation_errors);
        assert!(indentation_errors[0].message.contains("Use 2 (not 4)"));
    }

    #[test]
    fn test_incorrect_indentation_in_def_too_few_spaces() {
        let source = r#"
def foo
 bar
end
"#;
        let diagnostics = check_source(source);
        let indentation_errors: Vec<_> = diagnostics.iter().filter(|d| d.rule_id == super::RULE_ID).collect();
        assert_eq!(indentation_errors.len(), 1, "Expected 1 indentation error, got: {:?}", indentation_errors);
        assert!(indentation_errors[0].message.contains("Use 2 (not 1)"));
    }

    #[test]
    fn test_no_indentation_in_def() {
        let source = r#"
def foo
bar
end
"#;
        let diagnostics = check_source(source);
        let indentation_errors: Vec<_> = diagnostics.iter().filter(|d| d.rule_id == super::RULE_ID).collect();
        assert_eq!(indentation_errors.len(), 1, "Expected 1 indentation error, got: {:?}", indentation_errors);
        assert!(indentation_errors[0].message.contains("Use 2 (not 0)"));
    }

    #[test]
    fn test_correct_indentation_in_class() {
        let source = r#"
class Foo
  def bar
    baz
  end
end
"#;
        let diagnostics = check_source(source);
        let indentation_errors: Vec<_> = diagnostics.iter().filter(|d| d.rule_id == super::RULE_ID).collect();
        assert!(indentation_errors.is_empty(), "Expected no indentation errors, got: {:?}", indentation_errors);
    }

    #[test]
    fn test_correct_indentation_in_if() {
        let source = r#"
if condition
  do_something
end
"#;
        let diagnostics = check_source(source);
        let indentation_errors: Vec<_> = diagnostics.iter().filter(|d| d.rule_id == super::RULE_ID).collect();
        assert!(indentation_errors.is_empty(), "Expected no indentation errors, got: {:?}", indentation_errors);
    }

    #[test]
    fn test_correct_indentation_in_while() {
        let source = r#"
while condition
  do_something
end
"#;
        let diagnostics = check_source(source);
        let indentation_errors: Vec<_> = diagnostics.iter().filter(|d| d.rule_id == super::RULE_ID).collect();
        assert!(indentation_errors.is_empty(), "Expected no indentation errors, got: {:?}", indentation_errors);
    }

    #[test]
    fn test_same_line_body_is_allowed() {
        // "else do_something" style - body on same line as keyword
        let source = r#"
if condition
  foo
else bar
end
"#;
        let diagnostics = check_source(source);
        let indentation_errors: Vec<_> = diagnostics.iter().filter(|d| d.rule_id == super::RULE_ID).collect();
        // Should not report error for "else bar" since it's on the same line
        assert!(
            indentation_errors.is_empty(),
            "Expected no indentation errors for same-line body, got: {:?}",
            indentation_errors
        );
    }

    #[test]
    fn test_empty_body_is_allowed() {
        let source = r#"
def foo
end
"#;
        let diagnostics = check_source(source);
        let indentation_errors: Vec<_> = diagnostics.iter().filter(|d| d.rule_id == super::RULE_ID).collect();
        assert!(
            indentation_errors.is_empty(),
            "Expected no indentation errors for empty body, got: {:?}",
            indentation_errors
        );
    }
}
