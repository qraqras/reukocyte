//! Layout/IndentationWidth cop
//!
//! Checks for consistent indentation width.
//!
//! # Examples
//!
//! ```ruby
//! # bad (3 spaces)
//! def foo
//!    bar
//! end
//!
//! # good (2 spaces)
//! def foo
//!   bar
//! end
//! ```
//!
//! # Usage with CompositeVisitor
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

use crate::composite_visitor::{AstContext, AstCop};
use precop_ast::{
    ruby_prism::{self, Location as PrismLocation},
    DefNode, ClassNode, ModuleNode, BlockNode, Visit,
};
use precop_core::cop::OffenseCollector;
use precop_core::offense::{Location, Offense, Severity};

/// Checks for consistent indentation width.
pub struct IndentationWidth<'pr> {
    /// Expected indentation width (default: 2)
    pub width: usize,

    /// Source code being checked
    source: &'pr str,

    /// Line start offsets for converting byte offset to line number
    line_starts: Vec<usize>,

    /// Offense collector
    collector: OffenseCollector,
}

impl<'pr> IndentationWidth<'pr> {
    pub fn new(source: &'pr str, file_path: &str) -> Self {
        let collector = OffenseCollector::new();
        collector.set_file_path(file_path);

        // Pre-compute line start offsets
        let mut line_starts = vec![0];
        for (i, c) in source.char_indices() {
            if c == '\n' {
                line_starts.push(i + 1);
            }
        }

        Self {
            width: 2,
            source,
            line_starts,
            collector,
        }
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn name(&self) -> &'static str {
        "Layout/IndentationWidth"
    }

    pub fn offenses(&self) -> Vec<Offense> {
        self.collector.offenses()
    }

    /// Convert byte offset to line number (1-indexed)
    fn offset_to_line(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset) {
            Ok(line) => line + 1,
            Err(line) => line,
        }
    }

    /// Get the line content at a given line number (1-indexed)
    fn get_line(&self, line_num: usize) -> Option<&str> {
        self.source.lines().nth(line_num.saturating_sub(1))
    }

    /// Calculate the indentation of a line (number of leading spaces)
    fn line_indentation(&self, line_num: usize) -> usize {
        self.get_line(line_num)
            .map(|line| line.len() - line.trim_start().len())
            .unwrap_or(0)
    }

    /// Get line number from a Prism location
    fn location_line(&self, loc: &PrismLocation<'pr>) -> usize {
        self.offset_to_line(loc.start_offset())
    }

    /// Check indentation of a body relative to its parent
    fn check_body_indentation(&self, parent_line: usize, body_line: usize) {
        if body_line <= parent_line {
            return;
        }

        let parent_indent = self.line_indentation(parent_line);
        let body_indent = self.line_indentation(body_line);
        let expected_indent = parent_indent + self.width;

        // Skip blank lines
        if let Some(line) = self.get_line(body_line) {
            if line.trim().is_empty() {
                return;
            }
        }

        if body_indent != expected_indent {
            self.collector.add_offense(
                Offense::new(
                    self.name(),
                    format!(
                        "Use {} spaces for indentation (found {}).",
                        self.width,
                        body_indent.saturating_sub(parent_indent)
                    ),
                    self.collector.file_path(),
                    Location::new(body_line, 1, body_indent),
                )
                .with_severity(Severity::Convention),
            );
        }
    }
}

impl<'pr> Visit<'pr> for IndentationWidth<'pr> {
    fn visit_def_node(&mut self, node: &DefNode<'pr>) {
        let def_loc = node.location();
        let def_line = self.location_line(&def_loc);

        // Check body indentation if body exists
        if let Some(body) = node.body() {
            let body_loc = body.location();
            let body_line = self.location_line(&body_loc);
            self.check_body_indentation(def_line, body_line);
        }

        // Continue visiting child nodes
        ruby_prism::visit_def_node(self, node);
    }

    fn visit_class_node(&mut self, node: &ClassNode<'pr>) {
        let class_loc = node.location();
        let class_line = self.location_line(&class_loc);

        // Check body indentation if body exists
        if let Some(body) = node.body() {
            let body_loc = body.location();
            let body_line = self.location_line(&body_loc);
            self.check_body_indentation(class_line, body_line);
        }

        // Continue visiting child nodes
        ruby_prism::visit_class_node(self, node);
    }

    fn visit_module_node(&mut self, node: &ModuleNode<'pr>) {
        let module_loc = node.location();
        let module_line = self.location_line(&module_loc);

        // Check body indentation if body exists
        if let Some(body) = node.body() {
            let body_loc = body.location();
            let body_line = self.location_line(&body_loc);
            self.check_body_indentation(module_line, body_line);
        }

        // Continue visiting child nodes
        ruby_prism::visit_module_node(self, node);
    }

    fn visit_block_node(&mut self, node: &BlockNode<'pr>) {
        let block_loc = node.location();
        let block_line = self.location_line(&block_loc);

        // Check body indentation if body exists
        if let Some(body) = node.body() {
            let body_loc = body.location();
            let body_line = self.location_line(&body_loc);

            // Only check multi-line blocks
            if body_line > block_line {
                self.check_body_indentation(block_line, body_line);
            }
        }

        // Continue visiting child nodes
        ruby_prism::visit_block_node(self, node);
    }
}

// =============================================================================
// IndentationWidthCop - AstCop implementation for CompositeVisitor
// =============================================================================

/// AstCop implementation for indentation width checking.
///
/// This struct implements the `AstCop` trait and can be registered
/// with a `CompositeVisitor` for efficient single-pass AST traversal.
pub struct IndentationWidthCop {
    /// Expected indentation width (default: 2)
    pub width: usize,
    /// Whether the cop is enabled
    pub enabled: bool,
}

impl IndentationWidthCop {
    /// Create a new cop with default settings (width=2)
    pub fn new() -> Self {
        Self {
            width: 2,
            enabled: true,
        }
    }

    /// Create a new cop with a specific indentation width
    pub fn with_width(width: usize) -> Self {
        Self {
            width,
            enabled: true,
        }
    }

    /// Check indentation of a body relative to its parent
    fn check_body_indentation(
        &self,
        ctx: &AstContext,
        file_path: &str,
        parent_line: usize,
        body_line: usize,
    ) -> Option<Offense> {
        if body_line <= parent_line {
            return None;
        }

        let parent_indent = ctx.line_indentation(parent_line);
        let body_indent = ctx.line_indentation(body_line);
        let expected_indent = parent_indent + self.width;

        // Skip blank lines
        if let Some(line) = ctx.get_line(body_line) {
            if line.trim().is_empty() {
                return None;
            }
        }

        if body_indent != expected_indent {
            Some(
                Offense::new(
                    self.name(),
                    format!(
                        "Use {} spaces for indentation (found {}).",
                        self.width,
                        body_indent.saturating_sub(parent_indent)
                    ),
                    file_path.to_string(),
                    Location::new(body_line, 1, body_indent),
                )
                .with_severity(Severity::Convention),
            )
        } else {
            None
        }
    }
}

impl Default for IndentationWidthCop {
    fn default() -> Self {
        Self::new()
    }
}

impl AstCop for IndentationWidthCop {
    fn name(&self) -> &'static str {
        "Layout/IndentationWidth"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn on_def_node(&self, ctx: &AstContext, node: &DefNode) -> Vec<Offense> {
        let def_loc = node.location();
        let def_line = ctx.location_line(&def_loc);

        if let Some(body) = node.body() {
            let body_loc = body.location();
            let body_line = ctx.location_line(&body_loc);

            if let Some(offense) = self.check_body_indentation(ctx, ctx.file_path, def_line, body_line) {
                return vec![offense];
            }
        }
        vec![]
    }

    fn on_class_node(&self, ctx: &AstContext, node: &ClassNode) -> Vec<Offense> {
        let class_loc = node.location();
        let class_line = ctx.location_line(&class_loc);

        if let Some(body) = node.body() {
            let body_loc = body.location();
            let body_line = ctx.location_line(&body_loc);

            if let Some(offense) = self.check_body_indentation(ctx, ctx.file_path, class_line, body_line) {
                return vec![offense];
            }
        }
        vec![]
    }

    fn on_module_node(&self, ctx: &AstContext, node: &ModuleNode) -> Vec<Offense> {
        let module_loc = node.location();
        let module_line = ctx.location_line(&module_loc);

        if let Some(body) = node.body() {
            let body_loc = body.location();
            let body_line = ctx.location_line(&body_loc);

            if let Some(offense) = self.check_body_indentation(ctx, ctx.file_path, module_line, body_line) {
                return vec![offense];
            }
        }
        vec![]
    }

    fn on_block_node(&self, ctx: &AstContext, node: &BlockNode) -> Vec<Offense> {
        let block_loc = node.location();
        let block_line = ctx.location_line(&block_loc);

        if let Some(body) = node.body() {
            let body_loc = body.location();
            let body_line = ctx.location_line(&body_loc);

            // Only check multi-line blocks
            if body_line > block_line {
                if let Some(offense) = self.check_body_indentation(ctx, ctx.file_path, block_line, body_line) {
                    return vec![offense];
                }
            }
        }
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composite_visitor::CompositeVisitor;
    use precop_ast::parse;

    // Tests for the standalone IndentationWidth visitor
    mod standalone {
        use super::*;

        #[test]
        fn test_correct_indentation() {
            let source = r#"
def hello
  puts "world"
end
"#;
            let result = parse(source.as_bytes());
            let mut cop = IndentationWidth::new(source, "test.rb");
            cop.visit(&result.node());

            assert!(cop.offenses().is_empty());
        }

        #[test]
        fn test_wrong_indentation() {
            let source = r#"
def hello
   puts "world"
end
"#;
            let result = parse(source.as_bytes());
            let mut cop = IndentationWidth::new(source, "test.rb");
            cop.visit(&result.node());

            let offenses = cop.offenses();
            assert_eq!(offenses.len(), 1);
            assert!(offenses[0].message.contains("Use 2 spaces"));
        }

        #[test]
        fn test_nested_indentation() {
            let source = r#"
class Foo
  def bar
    puts "baz"
  end
end
"#;
            let result = parse(source.as_bytes());
            let mut cop = IndentationWidth::new(source, "test.rb");
            cop.visit(&result.node());

            assert!(cop.offenses().is_empty());
        }
    }

    // Tests for IndentationWidthCop with CompositeVisitor
    mod with_composite_visitor {
        use super::*;

        #[test]
        fn test_correct_indentation() {
            let source = r#"
def hello
  puts "world"
end
"#;
            let result = parse(source.as_bytes());
            let mut visitor = CompositeVisitor::new(source, "test.rb");
            visitor.register(IndentationWidthCop::new());
            visitor.visit(&result.node());

            assert!(visitor.offenses().is_empty());
        }

        #[test]
        fn test_wrong_indentation() {
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
        fn test_nested_indentation() {
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
        fn test_custom_width() {
            let source = r#"
def hello
    puts "world"
end
"#;
            let result = parse(source.as_bytes());
            let mut visitor = CompositeVisitor::new(source, "test.rb");
            visitor.register(IndentationWidthCop::with_width(4));
            visitor.visit(&result.node());

            assert!(visitor.offenses().is_empty());
        }

        #[test]
        fn test_disabled_cop() {
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
}
