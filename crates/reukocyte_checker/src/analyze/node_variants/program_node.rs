#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ProgramNode;

/// Run lint rules over a [`ProgramNode`] syntax node.
pub(crate) fn program_node(node: &ProgramNode, checker: &mut Checker) {
    // TODO: Add rules for ProgramNode
}
