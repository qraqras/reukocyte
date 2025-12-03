#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::RetryNode;

/// Run lint rules over a [`RetryNode`] syntax node.
pub(crate) fn retry_node(node: &RetryNode, checker: &mut Checker) {
    // TODO: Add rules for RetryNode
}
