#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ClassNode;

/// Run lint rules over a [`ClassNode`] syntax node.
pub(crate) fn class_node(node: &ClassNode, checker: &mut Checker) {
    // TODO: Add rules for ClassNode
}
