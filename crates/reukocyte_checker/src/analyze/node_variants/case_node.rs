#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::CaseNode;

/// Run lint rules over a [`CaseNode`] syntax node.
pub(crate) fn case_node(node: &CaseNode, checker: &mut Checker) {
    // TODO: Add rules for CaseNode
}
