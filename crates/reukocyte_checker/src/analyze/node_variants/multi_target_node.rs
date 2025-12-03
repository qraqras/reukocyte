#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::MultiTargetNode;

/// Run lint rules over a [`MultiTargetNode`] syntax node.
pub(crate) fn multi_target_node(node: &MultiTargetNode, checker: &mut Checker) {
    // TODO: Add rules for MultiTargetNode
}
