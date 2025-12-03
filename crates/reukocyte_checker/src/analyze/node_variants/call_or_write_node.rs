#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::CallOrWriteNode;

/// Run lint rules over a [`CallOrWriteNode`] syntax node.
pub(crate) fn call_or_write_node(node: &CallOrWriteNode, checker: &mut Checker) {
    // TODO: Add rules for CallOrWriteNode
}
