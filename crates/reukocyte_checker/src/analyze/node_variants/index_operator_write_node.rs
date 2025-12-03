#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::IndexOperatorWriteNode;

/// Run lint rules over a [`IndexOperatorWriteNode`] syntax node.
pub(crate) fn index_operator_write_node(node: &IndexOperatorWriteNode, checker: &mut Checker) {
    // TODO: Add rules for IndexOperatorWriteNode
}
