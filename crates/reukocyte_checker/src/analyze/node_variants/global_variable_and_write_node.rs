#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::GlobalVariableAndWriteNode;

/// Run lint rules over a [`GlobalVariableAndWriteNode`] syntax node.
pub(crate) fn global_variable_and_write_node(node: &GlobalVariableAndWriteNode, checker: &mut Checker) {
    // TODO: Add rules for GlobalVariableAndWriteNode
}
