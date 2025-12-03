#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::GlobalVariableOrWriteNode;

/// Run lint rules over a [`GlobalVariableOrWriteNode`] syntax node.
pub(crate) fn global_variable_or_write_node(node: &GlobalVariableOrWriteNode, checker: &mut Checker) {
    // TODO: Add rules for GlobalVariableOrWriteNode
}
