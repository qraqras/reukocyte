#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::PostExecutionNode;

/// Run lint rules over a [`PostExecutionNode`] syntax node.
pub(crate) fn post_execution_node(node: &PostExecutionNode, checker: &mut Checker) {
    // TODO: Add rules for PostExecutionNode
}
