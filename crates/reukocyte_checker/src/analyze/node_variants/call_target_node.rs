#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::CallTargetNode;

/// Run lint rules over a [`CallTargetNode`] syntax node.
pub(crate) fn call_target_node(node: &CallTargetNode, checker: &mut Checker) {
    // TODO: Add rules for CallTargetNode
}
