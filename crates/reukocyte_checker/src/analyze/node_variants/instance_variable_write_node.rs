#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::InstanceVariableWriteNode;

/// Run lint rules over a [`InstanceVariableWriteNode`] syntax node.
pub(crate) fn instance_variable_write_node(node: &InstanceVariableWriteNode, checker: &mut Checker) {
    // TODO: Add rules for InstanceVariableWriteNode
}
