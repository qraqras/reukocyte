#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::RequiredParameterNode;

/// Run lint rules over a [`RequiredParameterNode`] syntax node.
pub(crate) fn required_parameter_node(node: &RequiredParameterNode, checker: &mut Checker) {
    // TODO: Add rules for RequiredParameterNode
}
