#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::RedoNode;

/// Run lint rules over a [`RedoNode`] syntax node.
pub(crate) fn redo_node(node: &RedoNode, checker: &mut Checker) {
    // TODO: Add rules for RedoNode
}
