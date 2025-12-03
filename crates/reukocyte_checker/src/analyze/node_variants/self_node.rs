#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::SelfNode;

/// Run lint rules over a [`SelfNode`] syntax node.
pub(crate) fn self_node(node: &SelfNode, checker: &mut Checker) {
    // TODO: Add rules for SelfNode
}
