#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::LocalVariableOperatorWriteNode;

/// Run lint rules over a [`LocalVariableOperatorWriteNode`] syntax node.
pub(crate) fn local_variable_operator_write_node(node: &LocalVariableOperatorWriteNode, checker: &mut Checker) {
    // TODO: Add rules for LocalVariableOperatorWriteNode
}
