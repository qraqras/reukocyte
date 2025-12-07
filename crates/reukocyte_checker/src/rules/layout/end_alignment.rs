//! Layout/EndAlignment rule.
//!
//! Checks whether the `end` keyword is aligned properly.

#![allow(dead_code, unused_variables)]

use crate::checker::Checker;
use crate::config::layout::end_alignment::EnforcedStyleAlignWith;
use crate::rule::{Check, LayoutRule, Rule, RuleId};
use crate::utility::call_node::*;
use ruby_prism::StatementsNode;
use ruby_prism::*;

// ============================================================================
// EndAlignment Rule Definition
// ============================================================================

/// Layout/EndAlignment rule.
///
/// This rule checks whether the `end` keyword is aligned with the
/// corresponding keyword (`class`, `module`, `if`, etc.) or with the
/// start of the line.
pub struct EndAlignment;

impl Rule for EndAlignment {
    const ID: RuleId = RuleId::Layout(LayoutRule::EndAlignment);
}

impl Check<StatementsNode<'_>> for EndAlignment {
    fn check(node: &StatementsNode, checker: &mut Checker) {
        check_statements(node, checker);
    }
}

// ============================================================================
// Implementation
// ============================================================================

// Target nodes:
// ClassNode
// ModuleNode
// IfNode
// ElseNode (parent is IfNode)
// UnlessNode
// WhileNode
// UntilNode
// CaseNode
// CaseMatchNode
// Assignments

fn check_statements(statements: &StatementsNode, checker: &mut Checker) {
    let line_index = checker.line_index();
    if let Some(parent) = checker.parent() {
        match parent {
            Node::ClassNode { .. } => {
                let class_node = parent.as_class_node().unwrap();
                check_other_alignment(
                    &class_node.class_keyword_loc(),
                    line_index.line_start_offset(class_node.location().start_offset()),
                    checker,
                );
            }
            Node::SingletonClassNode { .. } => {
                // TODO: 代入文の右辺だった場合は、check_assignmentで処理すること
                let singleton_class_node = parent.as_singleton_class_node().unwrap();
                check_other_alignment(
                    &singleton_class_node.class_keyword_loc(),
                    line_index.line_start_offset(singleton_class_node.location().start_offset()),
                    checker,
                );
            }
            Node::ModuleNode { .. } => {
                let module_node = parent.as_module_node().unwrap();
                check_other_alignment(
                    &module_node.module_keyword_loc(),
                    line_index.line_start_offset(module_node.location().start_offset()),
                    checker,
                );
            }
            Node::IfNode { .. } => {
                let if_node = parent.as_if_node().unwrap();
                // If not ternary, check alignment of 'if' keyword
                if let Some(if_keyword_loc) = if_node.if_keyword_loc() {
                    check_other_alignment(&if_keyword_loc, line_index.line_start_offset(if_node.location().start_offset()), checker);
                }
            }
            Node::ElseNode { .. } => {}
            Node::UnlessNode { .. } => {
                let unless_node = parent.as_unless_node().unwrap();
                check_other_alignment(
                    &unless_node.keyword_loc(),
                    line_index.line_start_offset(unless_node.location().start_offset()),
                    checker,
                );
            }
            Node::WhileNode { .. } => {
                let while_node = parent.as_while_node().unwrap();
                check_other_alignment(
                    &while_node.keyword_loc(),
                    line_index.line_start_offset(while_node.location().start_offset()),
                    checker,
                );
            }
            Node::UntilNode { .. } => {
                let until_node = parent.as_until_node().unwrap();
                check_other_alignment(
                    &until_node.keyword_loc(),
                    line_index.line_start_offset(until_node.location().start_offset()),
                    checker,
                );
            }
            Node::CaseNode { .. } => {
                // TODO: 代入文の右辺だった場合は、check_assignmentで処理すること
                let case_node = parent.as_case_node().unwrap();
                check_other_alignment(
                    &case_node.case_keyword_loc(),
                    line_index.line_start_offset(case_node.location().start_offset()),
                    checker,
                );
            }
            Node::CaseMatchNode { .. } => {
                // TODO: 代入文の右辺だった場合は、check_assignmentで処理すること
                let case_match_node = parent.as_case_match_node().unwrap();
                check_other_alignment(
                    &case_match_node.case_keyword_loc(),
                    line_index.line_start_offset(case_match_node.location().start_offset()),
                    checker,
                );
            }
            _ => {}
        }
    }
}

pub fn on_assignment(node: Node, rhs: Node) {
    if let Some(rhs) = first_part_of_call_chain(rhs) {
        // if is_conditional(&rhs) ||
    }
}

fn check_other_alignment(keyword: &Location, start_of_line: usize, checker: &mut Checker) {}
fn check_asgn_alignment(node: &Node, keyword: &Location, rhs: &Node, checker: &mut Checker) {}
