use crate::checker::Checker;
use crate::config::EnforcedStyleAlignWith;
use crate::diagnostic::{Edit, Fix, Severity};
use crate::rule::{LayoutRule, RuleId};
use crate::utils::first_part_of_call_chain;
use ruby_prism::*;

/// Rule identifier for Layout/IndentationWidth.
pub const RULE_ID: RuleId = RuleId::Layout(LayoutRule::IndentationWidth);

/// Default indentation width (2 spaces).
pub const DEFAULT_WIDTH: i32 = 2;

/// Check if indented_internal_methods style is enabled.
/// TODO: Read from config (Layout/IndentationConsistency)
fn indented_internal_methods_style() -> bool {
    false
}

pub fn on_statements(statements: &StatementsNode, _base: bool, checker: &mut Checker) {
    let line_index = checker.line_index();
    let Some(parent) = checker.parent() else {
        return; // Top-level statements have no parent
    };
    match parent {
        Node::ElseNode { .. } => check_indentation(&parent.location(), statements, checker),
        Node::RescueNode { .. } => check_indentation(&parent.location(), statements, checker),
        Node::EnsureNode { .. } => check_indentation(&parent.location(), statements, checker),
        Node::ForNode { .. } => check_indentation(&parent.location(), statements, checker),
        Node::BeginNode { .. } => {
            let begin_node = parent.as_begin_node().unwrap();
            // Check if this is a standalone begin block (has begin keyword)
            if let Some(begin_keyword_loc) = begin_node.begin_keyword_loc() {
                // Use begin keyword as base for indentation
                check_indentation(&begin_keyword_loc, statements, checker);
            }
        }
        Node::BlockNode { .. } => {
            let closing_loc = parent.as_block_node().unwrap().closing_loc();
            if !line_index.begins_its_line(closing_loc.start_offset()) {
                check_indentation(&closing_loc, statements, checker);
            }
            if !indented_internal_methods_style() {
                check_members(&closing_loc, statements, checker);
            }
        }
        Node::ClassNode { .. } => {
            let class_keyword_loc = parent.as_class_node().unwrap().class_keyword_loc();
            if !line_index.in_same_line(class_keyword_loc.start_offset(), statements.location().start_offset()) {
                return;
            }
            check_members(&class_keyword_loc, statements, checker);
        }
        Node::CallNode { .. } => {
            // if let Some(call_node) = parent.as_call_node() {
            //     // Skip if method is an access modifier
            //     if !call_node.receiver().is_none() {
            //         return;
            //     }
            //     let def_end_config = &checker.config().layout.def_end_alignment;
            //     let style = def_end_config.enforced_style_align_with;
            //     let base = match style {
            //         EnforcedStyleAlignWith::Def => parent.location(), // DefNode
            //         _ => call_node.location(),                        // CallNode
            //     };
            //     check_indentation(&base, statements, checker);
            //     checker.ignore_node(&parent.location());
            //     return;
            // }
        }
        Node::DefNode { .. } => {
            if checker.is_ignored_node(parent.location().start_offset(), parent.location().end_offset()) {
                return;
            }
            check_indentation(&parent.location(), statements, checker);
        }
        Node::WhileNode { .. } => {
            let while_node = parent.as_while_node().unwrap();
            let start = while_node.location().start_offset();
            let end = while_node.location().end_offset();
            let keyword_loc = while_node.keyword_loc();
            let predicate_loc = while_node.predicate().location();
            if checker.is_ignored_node(start, end) {
                return;
            }
            if !line_index.in_same_line(keyword_loc.start_offset(), predicate_loc.end_offset()) {
                return;
            }
            check_indentation(&while_node.location(), statements, checker);
        }
        Node::UntilNode { .. } => {
            let until_node = parent.as_until_node().unwrap();
            let start = until_node.location().start_offset();
            let end = until_node.location().end_offset();
            let keyword_loc = until_node.keyword_loc();
            let predicate_loc = until_node.predicate().location();
            if checker.is_ignored_node(start, end) {
                return;
            }
            if !line_index.in_same_line(keyword_loc.start_offset(), predicate_loc.end_offset()) {
                return;
            }
            check_indentation(&until_node.location(), statements, checker);
        }
        Node::CaseNode { .. } => {
            let case_node = parent.as_case_node().unwrap();
            let mut last_when = None;
            for condition in case_node.conditions().iter() {
                if let Some(when_node) = condition.as_when_node()
                    && let Some(body) = when_node.statements()
                {
                    check_indentation(&when_node.location(), &body, checker);
                    last_when = Some(when_node);
                }
            }
            if let Some(last) = last_when
                && let Some(else_clause) = case_node.else_clause()
                && let Some(body) = else_clause.statements()
            {
                check_indentation(&last.location(), &body, checker);
            }
        }
        _ => {}
    }
}

/// ****************************************************************

/// Handle if node for indentation checking.
pub fn on_if(node: &IfNode, base_loc: Location, checker: &mut Checker) {}

/// Handle while node for indentation checking.
pub fn on_while(node: &WhileNode, base_loc: Location, checker: &mut Checker) {}

/// Handle until node for indentation checking.
pub fn on_until(node: &UntilNode, base_loc: Location, checker: &mut Checker) {}

/// Check assignment nodes for indentation width violations.
/// RuboCop equivalent: check_assignment(node, rhs)
pub fn check_assignment(_node: Node, rhs: Node, checker: &mut Checker) {
    let rhs = first_part_of_call_chain(rhs);
    if let Some(rhs) = rhs {
        let rhs_loc = rhs.location();
        checker.ignore_node(&rhs_loc);
    }
}

/// Check indentation of body relative to base.
/// RuboCop equivalent: check_indentation(base_loc, body_node, style = 'normal')
pub fn check_indentation(base_loc: &Location, body_node: &StatementsNode, checker: &mut Checker) {
    check_indentation_style(base_loc, body_node, "normal", checker);
}

/// Check indentation with a specific style name for error messages.
pub fn check_indentation_style(base_loc: &Location, body_node: &StatementsNode, style: &str, checker: &mut Checker) {
    if !indentation_to_check(base_loc, body_node, checker) {
        return;
    }

    let body_loc = body_node.location();
    let indentation = checker
        .line_index()
        .column_offset_between(base_loc.start_offset(), body_loc.start_offset());
    let configured_width = checker.config.layout.indentation_width.width as usize;
    let column_delta = configured_width as i32 - indentation as i32;

    if column_delta == 0 {
        return;
    }

    // Get the first statement for reporting (RuboCop reports on first child of begin-type)
    let first_body = body_node.body().iter().next();
    let report_node_loc = if let Some(first) = first_body {
        first.location()
    } else {
        body_loc
    };

    let style_name = if style == "normal" {
        String::new()
    } else {
        format!(" {}", style)
    };

    // Calculate the range to highlight (from indentation start to actual position)
    let report_start = report_node_loc.start_offset();
    let report_end = report_node_loc.end_offset();

    // Create fix: adjust indentation by column_delta
    let line_start = checker.line_index().line_start_offset(report_start);
    let fix = if column_delta > 0 {
        // Need to add spaces
        let spaces = " ".repeat(column_delta as usize);
        Some(Fix::safe(vec![Edit::insertion(line_start, spaces)]))
    } else {
        // Need to remove spaces
        let remove_count = (-column_delta) as usize;
        let end = line_start + remove_count.min(report_start - line_start);
        Some(Fix::safe(vec![Edit::deletion(line_start, end)]))
    };

    checker.report(
        RULE_ID,
        format!(
            "Use {} (not {}) spaces for{} indentation.",
            configured_width, indentation, style_name
        ),
        Severity::Convention,
        report_start,
        report_end,
        fix,
    );
}

/// Check if indentation should be verified.
/// RuboCop equivalent: indentation_to_check?(base_loc, body_node)
fn indentation_to_check(base_loc: &Location, body_node: &StatementsNode, checker: &Checker) -> bool {
    true
}

/// Determine if the check should be skipped.
/// RuboCop equivalent: skip_check?(base_loc, body_node)
fn skip_check(base_loc: &Location, body_node: &StatementsNode, checker: &Checker) -> bool {
    let line_index = checker.line_index();
    let body_loc = body_node.location();

    // Skip if body is empty
    if body_node.body().iter().next().is_none() {
        return true;
    }

    // Don't check if expression is on same line as base
    if line_index.in_same_line(base_loc.start_offset(), body_loc.start_offset()) {
        return true;
    }

    // Don't check if body starts with access modifier
    if starts_with_access_modifier(body_node) {
        return true;
    }

    // Don't check indentation if the line doesn't start with the body
    // (e.g., lines like "else do_something")
    if let Some(first) = body_node.body().iter().next() {
        let first_loc = first.location();
        if !line_index.begins_its_line(first_loc.start_offset()) {
            return true;
        }
    }

    false
}

pub fn check_members(_base_loc: &Location, _members: &StatementsNode, _checker: &mut Checker) {
    // TODO: Implement check_members for class/module body indentation
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
        assert!(
            indentation_errors.is_empty(),
            "Expected no indentation errors, got: {:?}",
            indentation_errors
        );
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
        assert_eq!(
            indentation_errors.len(),
            1,
            "Expected 1 indentation error, got: {:?}",
            indentation_errors
        );
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
        assert_eq!(
            indentation_errors.len(),
            1,
            "Expected 1 indentation error, got: {:?}",
            indentation_errors
        );
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
        assert_eq!(
            indentation_errors.len(),
            1,
            "Expected 1 indentation error, got: {:?}",
            indentation_errors
        );
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
        assert!(
            indentation_errors.is_empty(),
            "Expected no indentation errors, got: {:?}",
            indentation_errors
        );
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
        assert!(
            indentation_errors.is_empty(),
            "Expected no indentation errors, got: {:?}",
            indentation_errors
        );
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
        assert!(
            indentation_errors.is_empty(),
            "Expected no indentation errors, got: {:?}",
            indentation_errors
        );
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
