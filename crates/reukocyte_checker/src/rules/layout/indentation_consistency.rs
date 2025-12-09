use crate::checker::Checker;
use crate::config::layout::indentation_consistency::EnforcedStyle;
use crate::rule::Check;
use crate::rule::LayoutRule;
use crate::rule::Rule;
use crate::rule::RuleId;
use crate::utility::access_modifier::is_bare_access_modifier;
use crate::utility::alignment::check_alignment;
use reukocyte_macros::check;
use ruby_prism::*;

/// Layout/IndentationConsistency rule.
pub struct IndentationConsistency;
impl Rule for IndentationConsistency {
    const ID: RuleId = RuleId::Layout(LayoutRule::IndentationConsistency);
}
#[check(StatementsNode)]
impl Check<StatementsNode<'_>> for IndentationConsistency {
    fn check(node: &StatementsNode, checker: &mut Checker) {
        match checker.config().layout.indentation_consistency.enforced_style {
            EnforcedStyle::Normal => check_normal_style(node, checker),
            EnforcedStyle::IndentedInternalMethods => check_indented_internal_methods_style(node, checker),
        }
    }
}

/// Check indentation consistency in normal style.
fn check_normal_style(node: &StatementsNode, checker: &mut Checker) {
    let targets = node
        .body()
        .iter()
        .filter_map(|child| {
            let node_id = checker.semantic().node_id_for(&child)?;
            if !is_bare_access_modifier(&node_id, checker) {
                Some(child.location())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    check_alignment(targets, base_column_for_normal_style(node, checker), IndentationConsistency::ID, checker);
}

/// Check indentation consistency in indented internal methods style.
fn check_indented_internal_methods_style(node: &StatementsNode, checker: &mut Checker) {
    let mut children_to_check = Vec::new();
    for statement in node.body().iter() {
        let Some(node_id) = checker.semantic().node_id_for(&statement) else {
            continue;
        };
        if is_bare_access_modifier(&node_id, checker) {
            children_to_check.push(Vec::new());
        } else {
            if let Some(last_group) = children_to_check.last_mut() {
                last_group.push(statement.location());
            }
        }
    }
    for group in children_to_check {
        check_alignment(group, None, IndentationConsistency::ID, checker);
    }
}

/// Determine the base column for normal style indentation checking.
fn base_column_for_normal_style(node: &StatementsNode, checker: &mut Checker) -> Option<usize> {
    let first_child = node.body().iter().next();
    if let Some(first_child) = first_child
        && let Some(node_id) = checker.semantic().node_id_for(&first_child)
        && is_bare_access_modifier(&node_id, checker)
    {
        let access_modifier_indent = checker.line_index().column_number(first_child.location().start_offset());
        // If the StatementsNode is inside a module/class, ensure access modifier is more indented
        if let Some(parent) = checker.semantic().parent() {
            let module_indent = checker.line_index().column_number(parent.location().start_offset());
            if module_indent < access_modifier_indent {
                return Some(access_modifier_indent);
            }
        } else {
            return Some(access_modifier_indent);
        }
    }
    None
}
