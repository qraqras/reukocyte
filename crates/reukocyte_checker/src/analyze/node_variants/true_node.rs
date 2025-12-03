#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::TrueNode;

/// Run lint rules over a [`TrueNode`] syntax node.
pub(crate) fn true_node(node: &TrueNode, checker: &mut Checker) {
    // TODO: Add rules for TrueNode
}
