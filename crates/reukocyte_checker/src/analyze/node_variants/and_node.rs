#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::AndNode;

/// Run lint rules over a [`AndNode`] syntax node.
pub(crate) fn and_node(node: &AndNode, checker: &mut Checker) {
    // TODO: Add rules for AndNode
}
