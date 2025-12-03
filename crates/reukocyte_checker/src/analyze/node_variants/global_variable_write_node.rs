#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::GlobalVariableWriteNode;

/// Run lint rules over a [`GlobalVariableWriteNode`] syntax node.
pub(crate) fn global_variable_write_node(node: &GlobalVariableWriteNode, checker: &mut Checker) {
    // TODO: Add rules for GlobalVariableWriteNode
}
