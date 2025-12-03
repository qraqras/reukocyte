#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::IndexOrWriteNode;

/// Run lint rules over a [`IndexOrWriteNode`] syntax node.
pub(crate) fn index_or_write_node(node: &IndexOrWriteNode, checker: &mut Checker) {
    // TODO: Add rules for IndexOrWriteNode
}
