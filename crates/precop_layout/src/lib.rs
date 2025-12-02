//! PreCop Layout Cops
//!
//! Implementation of RuboCop's Layout cops in Rust.
//!
//! Each module corresponds to a RuboCop Layout cop:
//! - `trailing_whitespace` -> `Layout/TrailingWhitespace`
//! - `trailing_empty_lines` -> `Layout/TrailingEmptyLines`
//! - `indentation_width` -> `Layout/IndentationWidth`
//! - etc.
//!
//! ## Cop Types
//!
//! There are two types of cops:
//!
//! ### Token-based cops (implement `Cop` trait)
//! These cops work on raw source text without parsing:
//! - `TrailingWhitespace`
//! - `TrailingEmptyLines`
//!
//! ### AST-based cops (via `CompositeVisitor`)
//! These cops use the Visitor pattern to traverse the AST.
//! All AST-based cops run in a single traversal for efficiency:
//! - `IndentationWidth`
//!
//! ## Usage
//!
//! ```ignore
//! use precop_layout::{CompositeVisitor, TrailingWhitespace};
//! use precop_ast::{parse, Visit};
//!
//! let source = "def foo\n  bar\nend\n";
//!
//! // Token-based cops
//! let cop = TrailingWhitespace::new();
//! let offenses = cop.check(&context);
//!
//! // AST-based cops (single traversal for all)
//! let result = parse(source.as_bytes());
//! let mut visitor = CompositeVisitor::new(source, "test.rb");
//! visitor.register(IndentationWidthCop::new());
//! visitor.visit(&result.node());
//! let offenses = visitor.offenses();
//! ```

pub mod trailing_whitespace;
pub mod trailing_empty_lines;
pub mod indentation_width;
pub mod composite_visitor;

pub use trailing_whitespace::TrailingWhitespace;
pub use trailing_empty_lines::TrailingEmptyLines;
pub use indentation_width::{IndentationWidth, IndentationWidthCop};
pub use composite_visitor::{CompositeVisitor, AstContext, AstCop};

use precop_core::Cop;

/// Returns all available token-based Layout cops
pub fn all_cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(TrailingWhitespace::new()),
        Box::new(TrailingEmptyLines::default()),
    ]
}

/// Returns all available AST-based Layout cops
pub fn all_ast_cops() -> Vec<Box<dyn AstCop>> {
    vec![
        Box::new(IndentationWidthCop::new()),
    ]
}
