#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::InstanceVariableAndWriteNode;

/// Run lint rules over a [`InstanceVariableAndWriteNode`] syntax node.
pub(crate) fn instance_variable_and_write_node(node: &InstanceVariableAndWriteNode, checker: &mut Checker) {
    // TODO: Add rules for InstanceVariableAndWriteNode
}
