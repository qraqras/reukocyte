#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::HashNode;

/// Run lint rules over a [`HashNode`] syntax node.
pub(crate) fn hash_node(node: &HashNode, checker: &mut Checker) {
    // TODO: Add rules for HashNode
}
