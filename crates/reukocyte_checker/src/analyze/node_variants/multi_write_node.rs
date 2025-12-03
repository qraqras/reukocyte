#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::MultiWriteNode;

/// Run lint rules over a [`MultiWriteNode`] syntax node.
pub(crate) fn multi_write_node(node: &MultiWriteNode, checker: &mut Checker) {
    // TODO: Add rules for MultiWriteNode
}
