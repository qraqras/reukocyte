#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ClassVariableReadNode;

/// Run lint rules over a [`ClassVariableReadNode`] syntax node.
pub(crate) fn class_variable_read_node(node: &ClassVariableReadNode, checker: &mut Checker) {
    // TODO: Add rules for ClassVariableReadNode
}
