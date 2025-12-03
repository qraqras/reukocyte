#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::RangeNode;

/// Run lint rules over a [`RangeNode`] syntax node.
pub(crate) fn range_node(node: &RangeNode, checker: &mut Checker) {
    // TODO: Add rules for RangeNode
}
