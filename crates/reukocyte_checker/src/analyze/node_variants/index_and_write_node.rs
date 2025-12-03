#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::IndexAndWriteNode;

/// Run lint rules over a [`IndexAndWriteNode`] syntax node.
pub(crate) fn index_and_write_node(node: &IndexAndWriteNode, checker: &mut Checker) {
    // TODO: Add rules for IndexAndWriteNode
}
