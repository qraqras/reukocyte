#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ConstantPathTargetNode;

/// Run lint rules over a [`ConstantPathTargetNode`] syntax node.
pub(crate) fn constant_path_target_node(node: &ConstantPathTargetNode, checker: &mut Checker) {
    // TODO: Add rules for ConstantPathTargetNode
}
