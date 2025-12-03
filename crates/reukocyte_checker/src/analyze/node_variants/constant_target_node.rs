#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ConstantTargetNode;

/// Run lint rules over a [`ConstantTargetNode`] syntax node.
pub(crate) fn constant_target_node(node: &ConstantTargetNode, checker: &mut Checker) {
    // TODO: Add rules for ConstantTargetNode
}
