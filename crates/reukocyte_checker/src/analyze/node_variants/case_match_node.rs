#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::CaseMatchNode;

/// Run lint rules over a [`CaseMatchNode`] syntax node.
pub(crate) fn case_match_node(node: &CaseMatchNode, checker: &mut Checker) {
    // TODO: Add rules for CaseMatchNode
}
