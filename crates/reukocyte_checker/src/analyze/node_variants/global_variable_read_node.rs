#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::GlobalVariableReadNode;

/// Run lint rules over a [`GlobalVariableReadNode`] syntax node.
pub(crate) fn global_variable_read_node(node: &GlobalVariableReadNode, checker: &mut Checker) {
    // TODO: Add rules for GlobalVariableReadNode
}
