#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ForwardingParameterNode;

/// Run lint rules over a [`ForwardingParameterNode`] syntax node.
pub(crate) fn forwarding_parameter_node(node: &ForwardingParameterNode, checker: &mut Checker) {
    // TODO: Add rules for ForwardingParameterNode
}
