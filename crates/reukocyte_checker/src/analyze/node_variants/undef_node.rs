#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::UndefNode;

/// Run lint rules over a [`UndefNode`] syntax node.
pub(crate) fn undef_node(node: &UndefNode, checker: &mut Checker) {
    // TODO: Add rules for UndefNode
}
