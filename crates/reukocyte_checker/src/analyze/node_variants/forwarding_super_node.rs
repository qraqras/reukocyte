#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ForwardingSuperNode;

/// Run lint rules over a [`ForwardingSuperNode`] syntax node.
pub(crate) fn forwarding_super_node(node: &ForwardingSuperNode, checker: &mut Checker) {
    // TODO: Add rules for ForwardingSuperNode
}
