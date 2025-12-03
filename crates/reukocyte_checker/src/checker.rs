use crate::analyze;
use crate::locator::LineIndex;
use crate::rule::RuleId;
use crate::{Diagnostic, Fix, Severity};
use ruby_prism::Visit;
use std::cell::OnceCell;

/// The main checker that traverses the AST and runs rules.
pub struct Checker<'rk> {
    source: &'rk [u8],
    line_index: OnceCell<LineIndex>,
    diagnostics: Vec<Diagnostic>,
}

impl<'rk> Checker<'rk> {
    pub fn new(source: &'rk [u8]) -> Self {
        Self {
            source: source,
            line_index: OnceCell::new(),
            diagnostics: Vec::new(),
        }
    }
    pub fn source(&self) -> &[u8] {
        self.source
    }
    pub fn line_index(&self) -> &LineIndex {
        self.line_index
            .get_or_init(|| LineIndex::from_source(self.source))
    }
    pub fn push_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Create and push a diagnostic. Line/column are calculated from offsets.
    pub fn report(
        &mut self,
        rule_id: RuleId,
        message: String,
        severity: Severity,
        start_offset: usize,
        end_offset: usize,
        fix: Option<Fix>,
    ) {
        let index = self
            .line_index
            .get_or_init(|| LineIndex::from_source(self.source));
        let (line_start, column_start) = index.line_column(start_offset);
        let (line_end, column_end) = index.line_column(end_offset);

        self.diagnostics.push(Diagnostic::new(
            rule_id,
            message,
            severity,
            start_offset,
            end_offset,
            line_start,
            line_end,
            column_start,
            column_end,
            fix,
        ));
    }

    pub fn into_diagnostics(mut self) -> Vec<Diagnostic> {
        self.diagnostics
            .sort_by_key(|d| (d.line_start, d.column_start));
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
