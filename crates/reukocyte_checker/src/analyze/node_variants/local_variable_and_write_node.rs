#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::LocalVariableAndWriteNode;

/// Run lint rules over a [`LocalVariableAndWriteNode`] syntax node.
pub(crate) fn local_variable_and_write_node(node: &LocalVariableAndWriteNode, checker: &mut Checker) {
    // TODO: Add rules for LocalVariableAndWriteNode
}
