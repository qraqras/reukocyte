#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::RescueNode;

/// Run lint rules over a [`RescueNode`] syntax node.
pub(crate) fn rescue_node(node: &RescueNode, checker: &mut Checker) {
    // TODO: Add rules for RescueNode
}
