#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::BeginNode;

/// Run lint rules over a [`BeginNode`] syntax node.
pub(crate) fn begin_node(node: &BeginNode, checker: &mut Checker) {
    // TODO: Add rules for BeginNode
}
