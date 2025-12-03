#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::BreakNode;

/// Run lint rules over a [`BreakNode`] syntax node.
pub(crate) fn break_node(node: &BreakNode, checker: &mut Checker) {
    // TODO: Add rules for BreakNode
}
