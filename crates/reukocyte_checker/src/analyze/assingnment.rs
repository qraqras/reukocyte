#![allow(unused_variables)]
use crate::checker::Checker;
use crate::rules::layout::indentation_width;
use ruby_prism::*;

/// Run lint rules over a [`CallAndWriteNode`] syntax node.
/// Run lint rules over a [`CallOperatorWriteNode`] syntax node.
/// Run lint rules over a [`CallOrWriteNode`] syntax node.
/// Run lint rules over a [`ClassVariableAndWriteNode`] syntax node.
/// Run lint rules over a [`ClassVariableOperatorWriteNode`] syntax node.
/// Run lint rules over a [`ClassVariableOrWriteNode`] syntax node.
/// Run lint rules over a [`ClassVariableWriteNode`] syntax node.
/// Run lint rules over a [`ConstantAndWriteNode`] syntax node.
/// Run lint rules over a [`ConstantOperatorWriteNode`] syntax node.
/// Run lint rules over a [`ConstantOrWriteNode`] syntax node.
/// Run lint rules over a [`ConstantPathAndWriteNode`] syntax node.
/// Run lint rules over a [`ConstantPathOperatorWriteNode`] syntax node.
/// Run lint rules over a [`ConstantPathOrWriteNode`] syntax node.
/// Run lint rules over a [`ConstantPathWriteNode`] syntax node.
/// Run lint rules over a [`ConstantWriteNode`] syntax node.
/// Run lint rules over a [`GlobalVariableAndWriteNode`] syntax node.
/// Run lint rules over a [`GlobalVariableOperatorWriteNode`] syntax node.
/// Run lint rules over a [`GlobalVariableOrWriteNode`] syntax node.
/// Run lint rules over a [`GlobalVariableWriteNode`] syntax node.
/// Run lint rules over a [`IndexAndWriteNode`] syntax node.
/// Run lint rules over a [`IndexOperatorWriteNode`] syntax node.
/// Run lint rules over a [`IndexOrWriteNode`] syntax node.
/// Run lint rules over a [`InstanceVariableAndWriteNode`] syntax node.
/// Run lint rules over a [`InstanceVariableOperatorWriteNode`] syntax node.
/// Run lint rules over a [`InstanceVariableOrWriteNode`] syntax node.
/// Run lint rules over a [`InstanceVariableWriteNode`] syntax node.
/// Run lint rules over a [`LocalVariableAndWriteNode`] syntax node.
/// Run lint rules over a [`LocalVariableOperatorWriteNode`] syntax node.
/// Run lint rules over a [`LocalVariableOrWriteNode`] syntax node.
/// Run lint rules over a [`LocalVariableWriteNode`] syntax node.
/// Run lint rules over a [`MatchWriteNode`] syntax node.
/// Run lint rules over a [`MultiWriteNode`] syntax node.
#[rustfmt::skip]
pub(crate) fn assignment(node: &Node, rhs: &Node, checker: &mut Checker) {
    indentation_width::check_assignment(node, rhs, checker);
}
