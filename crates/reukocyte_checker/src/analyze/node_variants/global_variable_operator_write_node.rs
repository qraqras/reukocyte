#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::GlobalVariableOperatorWriteNode;

/// Run lint rules over a [`GlobalVariableOperatorWriteNode`] syntax node.
pub(crate) fn global_variable_operator_write_node(node: &GlobalVariableOperatorWriteNode, checker: &mut Checker) {
    // TODO: Add rules for GlobalVariableOperatorWriteNode
}
