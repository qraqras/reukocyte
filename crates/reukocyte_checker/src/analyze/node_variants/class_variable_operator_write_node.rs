#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ClassVariableOperatorWriteNode;

/// Run lint rules over a [`ClassVariableOperatorWriteNode`] syntax node.
pub(crate) fn class_variable_operator_write_node(node: &ClassVariableOperatorWriteNode, checker: &mut Checker) {
    // TODO: Add rules for ClassVariableOperatorWriteNode
}
