#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::InstanceVariableOrWriteNode;

/// Run lint rules over a [`InstanceVariableOrWriteNode`] syntax node.
pub(crate) fn instance_variable_or_write_node(node: &InstanceVariableOrWriteNode, checker: &mut Checker) {
    // TODO: Add rules for InstanceVariableOrWriteNode
}
