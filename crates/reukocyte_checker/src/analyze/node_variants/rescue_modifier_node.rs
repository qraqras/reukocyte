#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::RescueModifierNode;

/// Run lint rules over a [`RescueModifierNode`] syntax node.
pub(crate) fn rescue_modifier_node(node: &RescueModifierNode, checker: &mut Checker) {
    // TODO: Add rules for RescueModifierNode
}
