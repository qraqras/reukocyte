#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::YieldNode;

/// Run lint rules over a [`YieldNode`] syntax node.
pub(crate) fn yield_node(node: &YieldNode, checker: &mut Checker) {
    // TODO: Add rules for YieldNode
}
