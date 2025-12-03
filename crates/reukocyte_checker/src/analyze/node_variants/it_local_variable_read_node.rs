#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ItLocalVariableReadNode;

/// Run lint rules over a [`ItLocalVariableReadNode`] syntax node.
pub(crate) fn it_local_variable_read_node(node: &ItLocalVariableReadNode, checker: &mut Checker) {
    // TODO: Add rules for ItLocalVariableReadNode
}
