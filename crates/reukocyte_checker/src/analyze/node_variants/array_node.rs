#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ArrayNode;

/// Run lint rules over a [`ArrayNode`] syntax node.
pub(crate) fn array_node(node: &ArrayNode, checker: &mut Checker) {
    // TODO: Add rules for ArrayNode
}
