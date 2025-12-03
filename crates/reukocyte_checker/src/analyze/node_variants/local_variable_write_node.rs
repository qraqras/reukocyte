#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::LocalVariableWriteNode;

/// Run lint rules over a [`LocalVariableWriteNode`] syntax node.
pub(crate) fn local_variable_write_node(node: &LocalVariableWriteNode, checker: &mut Checker) {
    // TODO: Add rules for LocalVariableWriteNode
}
