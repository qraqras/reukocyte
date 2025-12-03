#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ForwardingArgumentsNode;

/// Run lint rules over a [`ForwardingArgumentsNode`] syntax node.
pub(crate) fn forwarding_arguments_node(node: &ForwardingArgumentsNode, checker: &mut Checker) {
    // TODO: Add rules for ForwardingArgumentsNode
}
