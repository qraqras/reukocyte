#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ParametersNode;

/// Run lint rules over a [`ParametersNode`] syntax node.
pub(crate) fn parameters_node(node: &ParametersNode, checker: &mut Checker) {
    // TODO: Add rules for ParametersNode
}
