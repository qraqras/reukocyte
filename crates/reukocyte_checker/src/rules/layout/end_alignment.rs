use crate::checker::Checker;
use crate::config::layout::end_alignment::EnforcedStyleAlignWith;
use crate::custom_nodes::AssignmentNode;
use crate::custom_nodes::ConditionalNode;
use crate::diagnostic::Edit;
use crate::diagnostic::Fix;
use crate::rule::{Check, LayoutRule, Rule, RuleId};
use crate::utility::call_node::first_part_of_call_chain;
use crate::utility::node::if_conditional_node;
use crate::utility::node::is_assignment;
use reukocyte_macros::check;
use ruby_prism::*;

/// Get the config for this rule
#[inline]
fn config<'a>(checker: &'a Checker<'_>) -> &'a crate::config::layout::end_alignment::EndAlignment {
    &checker.config().layout.end_alignment
}

/// Layout/EndAlignment rule.
pub struct EndAlignment;
impl Rule for EndAlignment {
    const ID: RuleId = RuleId::Layout(LayoutRule::EndAlignment);
}
#[check(ClassNode)]
impl Check<ClassNode<'_>> for EndAlignment {
    fn check(node: &ClassNode, checker: &mut Checker) {
        check_other_alignment(node.location(), node.class_keyword_loc(), node.end_keyword_loc(), checker);
    }
}
#[check(SingletonClassNode)]
impl Check<SingletonClassNode<'_>> for EndAlignment {
    fn check(node: &SingletonClassNode, checker: &mut Checker) {
        if let Some(parent) = checker.semantic().parent()
            && is_assignment(parent)
        {
            check_asgn_alignment(parent.location(), node.location(), node.class_keyword_loc(), node.end_keyword_loc(), checker);
        } else {
            check_other_alignment(node.location(), node.class_keyword_loc(), node.end_keyword_loc(), checker);
        }
    }
}
#[check(ModuleNode)]
impl Check<ModuleNode<'_>> for EndAlignment {
    fn check(node: &ModuleNode, checker: &mut Checker) {
        check_other_alignment(node.location(), node.module_keyword_loc(), node.end_keyword_loc(), checker);
    }
}
#[check(IfNode)]
impl Check<IfNode<'_>> for EndAlignment {
    fn check(node: &IfNode, checker: &mut Checker) {
        // If not ternary, check alignment of 'if' keyword
        if let Some(if_keyword_loc) = node.if_keyword_loc()
            && let Some(end_keyword_loc) = node.end_keyword_loc()
        {
            check_other_alignment(node.location(), if_keyword_loc, end_keyword_loc, checker);
        }
    }
}
#[check(UnlessNode)]
impl Check<UnlessNode<'_>> for EndAlignment {
    fn check(node: &UnlessNode, checker: &mut Checker) {
        if let Some(end_keyword_loc) = node.end_keyword_loc() {
            check_other_alignment(node.location(), node.keyword_loc(), end_keyword_loc, checker);
        }
    }
}
#[check(WhileNode)]
impl Check<WhileNode<'_>> for EndAlignment {
    fn check(node: &WhileNode, checker: &mut Checker) {
        if let Some(closing_loc) = node.closing_loc() {
            check_other_alignment(node.location(), node.keyword_loc(), closing_loc, checker);
        }
    }
}
#[check(UntilNode)]
impl Check<UntilNode<'_>> for EndAlignment {
    fn check(node: &UntilNode, checker: &mut Checker) {
        if let Some(closing_loc) = node.closing_loc() {
            check_other_alignment(node.location(), node.keyword_loc(), closing_loc, checker);
        }
    }
}
#[check(CaseNode)]
impl Check<CaseNode<'_>> for EndAlignment {
    fn check(node: &CaseNode, checker: &mut Checker) {
        if let Some(parent) = checker.semantic().parent()
            && parent.as_arguments_node().is_some()
        {
            check_asgn_alignment(
                checker.semantic().ancestor(1).unwrap().location(),
                node.location(),
                node.case_keyword_loc(),
                node.end_keyword_loc(),
                checker,
            );
        } else {
            check_other_alignment(node.location(), node.case_keyword_loc(), node.end_keyword_loc(), checker);
        }
    }
}
#[check(CaseMatchNode)]
impl Check<CaseMatchNode<'_>> for EndAlignment {
    fn check(node: &CaseMatchNode, checker: &mut Checker) {
        if let Some(parent) = checker.semantic().parent()
            && parent.as_arguments_node().is_some()
        {
            check_asgn_alignment(
                checker.semantic().ancestor(1).unwrap().location(),
                node.location(),
                node.case_keyword_loc(),
                node.end_keyword_loc(),
                checker,
            );
        } else {
            check_other_alignment(node.location(), node.case_keyword_loc(), node.end_keyword_loc(), checker);
        }
    }
}
#[check(AssignmentNode)]
impl Check<AssignmentNode<'_>> for EndAlignment {
    fn check(node: &AssignmentNode, checker: &mut Checker) {
        if let Some(rhs) = first_part_of_call_chain(node.value()) {
            if_conditional_node(&rhs, |conditional_node| {
                if matches!(conditional_node, ConditionalNode::IfNode(n) if n.if_keyword_loc().is_none()) {
                    // Skip ternary if nodes
                    return;
                }
                if let Some(keyword_loc) = conditional_node.keyword_loc()
                    && let Some(end_keyword_loc) = conditional_node.end_keyword_loc()
                {
                    check_asgn_alignment(node.location(), conditional_node.location(), keyword_loc, end_keyword_loc, checker);
                }
            });
        }
    }
}

fn check_other_alignment(node_loc: Location, keyword_loc: Location, end_loc: Location, checker: &mut Checker) {
    check_end_kw_alignment(
        &node_loc,
        &end_loc,
        &keyword_loc,
        &keyword_loc,
        checker.line_index().line_start_offset(node_loc.start_offset()),
        checker,
    );
}

fn check_asgn_alignment(outer_loc: Location, inner_loc: Location, inner_keyword_loc: Location, inner_end_loc: Location, checker: &mut Checker) {
    check_end_kw_alignment(
        &outer_loc,
        &inner_end_loc,
        &inner_keyword_loc,
        match checker.line_index().are_on_same_line(outer_loc.start_offset(), inner_loc.start_offset()) {
            true => &outer_loc,
            false => &inner_loc,
        },
        checker.line_index().line_start_offset(inner_loc.start_offset()),
        checker,
    );
    checker.ignore_node(&inner_loc);
}

fn check_end_kw_alignment(loc: &Location, end_loc: &Location, keyword_loc: &Location, variable_loc: &Location, start_of_line: usize, checker: &mut Checker) {
    if checker.is_ignored_node(loc.start_offset(), loc.end_offset()) {
        return;
    }

    let line_index = checker.line_index();
    let column_delta = match checker.config().layout.end_alignment.enforced_style_align_with {
        EnforcedStyleAlignWith::Keyword => {
            let are_same_line = line_index.are_on_same_line(keyword_loc.start_offset(), end_loc.start_offset());
            if are_same_line {
                return;
            }
            let column_delta = line_index.column_offset_between(keyword_loc.start_offset(), end_loc.start_offset());
            if column_delta == 0 {
                return;
            }
            column_delta
        }
        EnforcedStyleAlignWith::Variable => {
            let are_same_line = line_index.are_on_same_line(end_loc.start_offset(), variable_loc.start_offset());
            if are_same_line {
                return;
            }
            let column_delta = line_index.column_offset_between(end_loc.start_offset(), variable_loc.start_offset());
            if column_delta == 0 {
                return;
            }
            column_delta
        }
        EnforcedStyleAlignWith::StartOfLine => {
            let are_same_line = line_index.are_on_same_line(end_loc.start_offset(), start_of_line);
            if are_same_line {
                return;
            }
            let column_delta = line_index.column_offset_between(end_loc.start_offset(), start_of_line);
            if column_delta == 0 {
                return;
            }
            column_delta
        }
    };
    let fix = if column_delta > 0 {
        let end = end_loc.start_offset();
        let start = end as i32 - column_delta;
        Fix::safe(vec![Edit::deletion(start as usize, end)])
    } else {
        Fix::safe(vec![Edit::insertion(end_loc.start_offset(), " ".repeat(column_delta.abs() as usize))])
    };

    checker.report(
        EndAlignment::ID,
        format!("`end` keyword should be aligned with its opening keyword."),
        config(checker).base.severity,
        end_loc.start_offset(),
        end_loc.end_offset(),
        Some(fix),
    );
}
