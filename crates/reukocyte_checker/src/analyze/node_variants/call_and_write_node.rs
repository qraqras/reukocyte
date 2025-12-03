#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::CallAndWriteNode;

/// Run lint rules over a [`CallAndWriteNode`] syntax node.
pub(crate) fn call_and_write_node(node: &CallAndWriteNode, checker: &mut Checker) {
    // TODO: Add rules for CallAndWriteNode
}
