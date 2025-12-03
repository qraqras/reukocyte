#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ClassVariableAndWriteNode;

/// Run lint rules over a [`ClassVariableAndWriteNode`] syntax node.
pub(crate) fn class_variable_and_write_node(node: &ClassVariableAndWriteNode, checker: &mut Checker) {
    // TODO: Add rules for ClassVariableAndWriteNode
}
