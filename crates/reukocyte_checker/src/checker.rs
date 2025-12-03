//! Checker - the main AST visitor that coordinates rule execution.
//!
//! Inspired by Ruff's Checker, this struct:
//! - Holds the source and diagnostics
//! - Implements the Visit trait for AST traversal
//! - Delegates to analyze module for rule dispatch

use std::cell::OnceCell;

use ruby_prism::Visit;

use crate::analyze;
use crate::locator::LineIndex;
use crate::Diagnostic;

/// The main checker that traverses the AST and runs rules.
pub struct Checker<'a> {
    /// The source code being checked
    source: &'a [u8],
    /// Line index for fast offset to location conversion (lazily initialized)
    line_index: OnceCell<LineIndex>,
    /// Collected diagnostics
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Checker<'a> {
    /// Create a new Checker for the given source.
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            line_index: OnceCell::new(),
            diagnostics: Vec::new(),
        }
    }

    /// Get the source code.
    pub fn source(&self) -> &[u8] {
        self.source
    }

    /// Push a diagnostic (Ruff-style API).
    pub fn push_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Convert offset to (line, column) using lazily-initialized line index.
    /// The line index is built on first call, then reused.
    /// Uses binary search for O(log n) performance.
    #[inline]
    pub fn offset_to_location(&self, offset: usize) -> (usize, usize) {
        self.line_index
            .get_or_init(|| LineIndex::from_source(self.source))
            .line_column(offset)
    }

    /// Consume the checker and return the diagnostics.
    pub fn into_diagnostics(mut self) -> Vec<Diagnostic> {
        // Sort by location
        self.diagnostics.sort_by_key(|d| (d.line, d.column));
        self.diagnostics
    }
}

impl Visit<'_> for Checker<'_> {
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode) {
        // Visit children first
        ruby_prism::visit_call_node(self, node);

        // Run rules via analyze module (Ruff-style)
        analyze::call_node(node, self);
    }
}
