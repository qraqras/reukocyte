#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::CallOperatorWriteNode;

/// Run lint rules over a [`CallOperatorWriteNode`] syntax node.
pub(crate) fn call_operator_write_node(node: &CallOperatorWriteNode, checker: &mut Checker) {
    // TODO: Add rules for CallOperatorWriteNode
}
