#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::CapturePatternNode;

/// Run lint rules over a [`CapturePatternNode`] syntax node.
pub(crate) fn capture_pattern_node(node: &CapturePatternNode, checker: &mut Checker) {
    // TODO: Add rules for CapturePatternNode
}
