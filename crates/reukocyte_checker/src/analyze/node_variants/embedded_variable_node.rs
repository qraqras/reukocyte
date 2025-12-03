#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::EmbeddedVariableNode;

/// Run lint rules over a [`EmbeddedVariableNode`] syntax node.
pub(crate) fn embedded_variable_node(node: &EmbeddedVariableNode, checker: &mut Checker) {
    // TODO: Add rules for EmbeddedVariableNode
}
