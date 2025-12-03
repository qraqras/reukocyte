#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::SingletonClassNode;

/// Run lint rules over a [`SingletonClassNode`] syntax node.
pub(crate) fn singleton_class_node(node: &SingletonClassNode, checker: &mut Checker) {
    // TODO: Add rules for SingletonClassNode
}
