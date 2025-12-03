#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ShareableConstantNode;

/// Run lint rules over a [`ShareableConstantNode`] syntax node.
pub(crate) fn shareable_constant_node(node: &ShareableConstantNode, checker: &mut Checker) {
    // TODO: Add rules for ShareableConstantNode
}
