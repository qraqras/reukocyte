#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ArgumentsNode;

/// Run lint rules over a [`ArgumentsNode`] syntax node.
pub(crate) fn arguments_node(node: &ArgumentsNode, checker: &mut Checker) {
    // TODO: Add rules for ArgumentsNode
}
