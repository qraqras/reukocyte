//! Composite Visitor for running multiple AST-based cops in a single traversal
//!
//! This module provides a way to run multiple cops that need AST traversal
//! in a single pass over the AST, similar to RuboCop's Commissioner.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │  CompositeVisitor                                       │
//! │  ├── cops: Vec<Box<dyn AstCop>>  ← 登録されたCop       │
//! │  └── impl Visit                                         │
//! │      └── visit_def_node()                               │
//! │          └── for cop in cops { cop.on_def_node() }     │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```ignore
//! use precop_layout::{CompositeVisitor, IndentationWidthCop};
//! use precop_ast::{parse, Visit};
//!
//! let source = "def foo\n  bar\nend\n";
//! let result = parse(source.as_bytes());
//!
//! let mut visitor = CompositeVisitor::new(source, "test.rb");
//! visitor.register(IndentationWidthCop::new());
//! visitor.visit(&result.node());
//!
//! let offenses = visitor.offenses();
//! ```

use precop_ast::ruby_prism::{self, Location as PrismLocation};
use precop_ast::{BlockNode, ClassNode, DefNode, ModuleNode, IfNode, Visit};
use precop_core::offense::Offense;
use std::cell::RefCell;

/// Context passed to AST cops during checking
pub struct AstContext<'a> {
    /// Source code being checked
    pub source: &'a str,
    /// File path being checked
    pub file_path: &'a str,
    /// Line start offsets for converting byte offset to line number
    line_starts: &'a [usize],
}

impl<'a> AstContext<'a> {
    /// Convert byte offset to line number (1-indexed)
    pub fn offset_to_line(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset) {
            Ok(line) => line + 1,
            Err(line) => line,
        }
    }

    /// Get the line content at a given line number (1-indexed)
    pub fn get_line(&self, line_num: usize) -> Option<&str> {
        self.source.lines().nth(line_num.saturating_sub(1))
    }

    /// Calculate the indentation of a line (number of leading spaces)
    pub fn line_indentation(&self, line_num: usize) -> usize {
        self.get_line(line_num)
            .map(|line| line.len() - line.trim_start().len())
            .unwrap_or(0)
    }

    /// Get line number from a Prism location
    pub fn location_line(&self, loc: &PrismLocation) -> usize {
        self.offset_to_line(loc.start_offset())
    }
}

/// Trait for AST-based cops that can be registered with CompositeVisitor.
///
/// Each cop implements the `on_*` methods for the node types it cares about.
/// The CompositeVisitor calls these methods during AST traversal.
pub trait AstCop: Send + Sync {
    /// Returns the name of the cop (e.g., "Layout/IndentationWidth")
    fn name(&self) -> &'static str;

    /// Returns whether the cop is enabled
    fn enabled(&self) -> bool {
        true
    }

    /// Called when visiting a def node
    fn on_def_node(&self, _ctx: &AstContext, _node: &DefNode) -> Vec<Offense> {
        vec![]
    }

    /// Called when visiting a class node
    fn on_class_node(&self, _ctx: &AstContext, _node: &ClassNode) -> Vec<Offense> {
        vec![]
    }

    /// Called when visiting a module node
    fn on_module_node(&self, _ctx: &AstContext, _node: &ModuleNode) -> Vec<Offense> {
        vec![]
    }

    /// Called when visiting a block node
    fn on_block_node(&self, _ctx: &AstContext, _node: &BlockNode) -> Vec<Offense> {
        vec![]
    }

    /// Called when visiting an if node
    fn on_if_node(&self, _ctx: &AstContext, _node: &IfNode) -> Vec<Offense> {
        vec![]
    }
}

/// A composite visitor that runs multiple AST-based cops in a single traversal.
///
/// This is similar to RuboCop's Commissioner pattern, where one AST walk
/// triggers checks for all enabled cops.
pub struct CompositeVisitor<'pr> {
    /// Source code being checked
    source: &'pr str,

    /// File path being checked
    file_path: String,

    /// Line start offsets for converting byte offset to line number
    line_starts: Vec<usize>,

    /// Registered AST cops
    cops: Vec<Box<dyn AstCop>>,

    /// Collected offenses from all cops
    offenses: RefCell<Vec<Offense>>,
}

impl<'pr> CompositeVisitor<'pr> {
    /// Create a new composite visitor
    pub fn new(source: &'pr str, file_path: &str) -> Self {
        // Pre-compute line start offsets
        let mut line_starts = vec![0];
        for (i, c) in source.char_indices() {
            if c == '\n' {
                line_starts.push(i + 1);
            }
        }

        Self {
            source,
            file_path: file_path.to_string(),
            line_starts,
            cops: Vec::new(),
            offenses: RefCell::new(Vec::new()),
        }
    }

    /// Register an AST cop
    pub fn register<C: AstCop + 'static>(&mut self, cop: C) -> &mut Self {
        self.cops.push(Box::new(cop));
        self
    }

    /// Register a boxed AST cop
    pub fn register_boxed(&mut self, cop: Box<dyn AstCop>) -> &mut Self {
        self.cops.push(cop);
        self
    }

    /// Register multiple AST cops
    pub fn register_all(&mut self, cops: Vec<Box<dyn AstCop>>) -> &mut Self {
        self.cops.extend(cops);
        self
    }

    /// Get all collected offenses
    pub fn offenses(&self) -> Vec<Offense> {
        self.offenses.borrow().clone()
    }

    /// Get the number of registered cops
    pub fn cop_count(&self) -> usize {
        self.cops.len()
    }

    /// Create the context for cops
    fn context(&self) -> AstContext<'_> {
        AstContext {
            source: self.source,
            file_path: &self.file_path,
            line_starts: &self.line_starts,
        }
    }

    /// Add offenses from a cop check
    fn collect_offenses(&self, offenses: Vec<Offense>) {
        self.offenses.borrow_mut().extend(offenses);
    }
}

impl<'pr> Visit<'pr> for CompositeVisitor<'pr> {
    fn visit_def_node(&mut self, node: &DefNode<'pr>) {
        let ctx = self.context();

        // Run all registered cops on this node
        for cop in &self.cops {
            if cop.enabled() {
                let offenses = cop.on_def_node(&ctx, node);
                self.collect_offenses(offenses);
            }
        }

        // Continue visiting child nodes
        ruby_prism::visit_def_node(self, node);
    }

    fn visit_class_node(&mut self, node: &ClassNode<'pr>) {
        let ctx = self.context();

        for cop in &self.cops {
            if cop.enabled() {
                let offenses = cop.on_class_node(&ctx, node);
                self.collect_offenses(offenses);
            }
        }

        ruby_prism::visit_class_node(self, node);
    }

    fn visit_module_node(&mut self, node: &ModuleNode<'pr>) {
        let ctx = self.context();

        for cop in &self.cops {
            if cop.enabled() {
                let offenses = cop.on_module_node(&ctx, node);
                self.collect_offenses(offenses);
            }
        }

        ruby_prism::visit_module_node(self, node);
    }

    fn visit_block_node(&mut self, node: &BlockNode<'pr>) {
        let ctx = self.context();

        for cop in &self.cops {
            if cop.enabled() {
                let offenses = cop.on_block_node(&ctx, node);
                self.collect_offenses(offenses);
            }
        }

        ruby_prism::visit_block_node(self, node);
    }

    fn visit_if_node(&mut self, node: &IfNode<'pr>) {
        let ctx = self.context();

        for cop in &self.cops {
            if cop.enabled() {
                let offenses = cop.on_if_node(&ctx, node);
                self.collect_offenses(offenses);
            }
        }

        ruby_prism::visit_if_node(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indentation_width::IndentationWidthCop;
    use precop_ast::parse;

    #[test]
    fn test_composite_visitor_with_registered_cop() {
        let source = r#"
def hello
   puts "world"
end
"#;
        let result = parse(source.as_bytes());
        let mut visitor = CompositeVisitor::new(source, "test.rb");
        visitor.register(IndentationWidthCop::new());
        visitor.visit(&result.node());

        let offenses = visitor.offenses();
        assert_eq!(offenses.len(), 1);
        assert!(offenses[0].message.contains("Use 2 spaces"));
        assert_eq!(offenses[0].cop_name, "Layout/IndentationWidth");
    }

    #[test]
    fn test_composite_visitor_no_cops_registered() {
        let source = r#"
def hello
   puts "world"
end
"#;
        let result = parse(source.as_bytes());
        let mut visitor = CompositeVisitor::new(source, "test.rb");
        // No cops registered
        visitor.visit(&result.node());

        assert!(visitor.offenses().is_empty());
        assert_eq!(visitor.cop_count(), 0);
    }

    #[test]
    fn test_composite_visitor_multiple_cops() {
        let source = r#"
def hello
   puts "world"
end
"#;
        let result = parse(source.as_bytes());
        let mut visitor = CompositeVisitor::new(source, "test.rb");

        // Register the same cop twice (for testing multiple cops)
        visitor.register(IndentationWidthCop::new());
        visitor.register(IndentationWidthCop::with_width(4));

        visitor.visit(&result.node());

        // Should get 2 offenses (one from each cop)
        assert_eq!(visitor.offenses().len(), 2);
        assert_eq!(visitor.cop_count(), 2);
    }

    #[test]
    fn test_composite_visitor_correct_code() {
        let source = r#"
class Foo
  def bar
    puts "baz"
  end
end
"#;
        let result = parse(source.as_bytes());
        let mut visitor = CompositeVisitor::new(source, "test.rb");
        visitor.register(IndentationWidthCop::new());
        visitor.visit(&result.node());

        assert!(visitor.offenses().is_empty());
    }

    #[test]
    fn test_composite_visitor_disabled_cop() {
        let source = r#"
def hello
   puts "world"
end
"#;
        let result = parse(source.as_bytes());
        let mut visitor = CompositeVisitor::new(source, "test.rb");

        let mut cop = IndentationWidthCop::new();
        cop.enabled = false;
        visitor.register(cop);

        visitor.visit(&result.node());

        // No offenses because cop is disabled
        assert!(visitor.offenses().is_empty());
    }
}
