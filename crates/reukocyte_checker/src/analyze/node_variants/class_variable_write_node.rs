#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ClassVariableWriteNode;

/// Run lint rules over a [`ClassVariableWriteNode`] syntax node.
pub(crate) fn class_variable_write_node(node: &ClassVariableWriteNode, checker: &mut Checker) {
    // TODO: Add rules for ClassVariableWriteNode
}
