//! Checker - the main AST visitor that coordinates rule execution.
//!
//! Inspired by Ruff's Checker, this struct:
//! - Holds the source and diagnostics
//! - Implements the Visit trait for AST traversal
//! - Calls analyze functions at each node

use ruby_prism::Visit;

use crate::analyze;
use crate::Diagnostic;

/// The main checker that traverses the AST and runs rules.
pub struct Checker<'a> {
    /// The source code being checked
    source: &'a [u8],
    /// Collected diagnostics
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Checker<'a> {
    /// Create a new Checker for the given source.
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            diagnostics: Vec::new(),
        }
    }

    /// Get the source code.
    pub fn source(&self) -> &[u8] {
        self.source
    }

    /// Add a diagnostic.
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Convert offset to (line, column).
    pub fn offset_to_location(&self, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut column = 1;

        for (i, &byte) in self.source.iter().enumerate() {
            if i >= offset {
                break;
            }
            if byte == b'\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        (line, column)
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
        // Step 2: Traversal (visit children first)
        ruby_prism::visit_call_node(self, node);

        // Step 4: Analysis (run rules)
        analyze::expression::call(self, node);
    }
}
