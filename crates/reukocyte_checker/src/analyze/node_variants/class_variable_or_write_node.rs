#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ClassVariableOrWriteNode;

/// Run lint rules over a [`ClassVariableOrWriteNode`] syntax node.
pub(crate) fn class_variable_or_write_node(node: &ClassVariableOrWriteNode, checker: &mut Checker) {
    // TODO: Add rules for ClassVariableOrWriteNode
}
