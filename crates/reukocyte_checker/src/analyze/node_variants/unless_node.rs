#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::UnlessNode;

/// Run lint rules over a [`UnlessNode`] syntax node.
pub(crate) fn unless_node(node: &UnlessNode, checker: &mut Checker) {
    // TODO: Add rules for UnlessNode
}
