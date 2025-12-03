#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::NextNode;

/// Run lint rules over a [`NextNode`] syntax node.
pub(crate) fn next_node(node: &NextNode, checker: &mut Checker) {
    // TODO: Add rules for NextNode
}
