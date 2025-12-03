#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::SplatNode;

/// Run lint rules over a [`SplatNode`] syntax node.
pub(crate) fn splat_node(node: &SplatNode, checker: &mut Checker) {
    // TODO: Add rules for SplatNode
}
