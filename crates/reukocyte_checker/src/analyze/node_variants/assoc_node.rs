#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::AssocNode;

/// Run lint rules over a [`AssocNode`] syntax node.
pub(crate) fn assoc_node(node: &AssocNode, checker: &mut Checker) {
    // TODO: Add rules for AssocNode
}
