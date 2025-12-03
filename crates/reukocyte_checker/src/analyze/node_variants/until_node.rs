#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::UntilNode;

/// Run lint rules over a [`UntilNode`] syntax node.
pub(crate) fn until_node(node: &UntilNode, checker: &mut Checker) {
    // TODO: Add rules for UntilNode
}
