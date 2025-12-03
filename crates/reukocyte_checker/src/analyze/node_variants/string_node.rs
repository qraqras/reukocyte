#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::StringNode;

/// Run lint rules over a [`StringNode`] syntax node.
pub(crate) fn string_node(node: &StringNode, checker: &mut Checker) {
    // TODO: Add rules for StringNode
}
