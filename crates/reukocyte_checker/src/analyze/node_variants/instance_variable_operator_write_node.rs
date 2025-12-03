#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::InstanceVariableOperatorWriteNode;

/// Run lint rules over a [`InstanceVariableOperatorWriteNode`] syntax node.
pub(crate) fn instance_variable_operator_write_node(node: &InstanceVariableOperatorWriteNode, checker: &mut Checker) {
    // TODO: Add rules for InstanceVariableOperatorWriteNode
}
