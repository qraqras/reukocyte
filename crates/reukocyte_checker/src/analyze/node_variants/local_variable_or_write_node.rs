#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::LocalVariableOrWriteNode;

/// Run lint rules over a [`LocalVariableOrWriteNode`] syntax node.
pub(crate) fn local_variable_or_write_node(node: &LocalVariableOrWriteNode, checker: &mut Checker) {
    // TODO: Add rules for LocalVariableOrWriteNode
}
