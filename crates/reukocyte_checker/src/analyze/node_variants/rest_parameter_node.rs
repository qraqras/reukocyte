#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::RestParameterNode;

/// Run lint rules over a [`RestParameterNode`] syntax node.
pub(crate) fn rest_parameter_node(node: &RestParameterNode, checker: &mut Checker) {
    // TODO: Add rules for RestParameterNode
}
