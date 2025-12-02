//! PreCop AST
//!
//! AST utilities for PreCop, wrapping ruby-prism for Ruby parsing.
//!
//! This module provides:
//! - Ruby source code parsing via Prism
//! - Re-exports of ruby_prism types for use in cops
//! - Visitor pattern support for AST traversal

pub use ruby_prism;

// Re-export commonly used types
pub use ruby_prism::{
    Node, ParseResult, Visit,
    // Node types commonly used in Layout cops
    ClassNode, DefNode, ModuleNode, BlockNode,
    BeginNode, IfNode, CaseNode, WhileNode, ForNode,
};

/// Parse Ruby source code into an AST
pub fn parse(source: &[u8]) -> ParseResult<'_> {
    ruby_prism::parse(source)
}

/// Check if the source code has any syntax errors
pub fn has_syntax_errors(source: &[u8]) -> bool {
    let result = parse(source);
    result.errors().count() > 0
}
