#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ForNode;

/// Run lint rules over a [`ForNode`] syntax node.
pub(crate) fn for_node(node: &ForNode, checker: &mut Checker) {
    // TODO: Add rules for ForNode
}
