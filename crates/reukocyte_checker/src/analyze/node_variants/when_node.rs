#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::WhenNode;

/// Run lint rules over a [`WhenNode`] syntax node.
pub(crate) fn when_node(node: &WhenNode, checker: &mut Checker) {
    // TODO: Add rules for WhenNode
}
