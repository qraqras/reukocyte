use crate::analyze;
use crate::config::Config;
use crate::diagnostic::RawDiagnostic;
use crate::locator::LineIndex;
use crate::rule::RuleId;
use crate::{Diagnostic, Fix, Severity};
use ruby_prism::{Location, Node, Visit};
use std::collections::HashSet;

/// The main checker that traverses the AST and runs rules.
pub struct Checker<'rk> {
    source: &'rk [u8],
    pub config: &'rk Config,
    pub line_index: LineIndex<'rk>,
    raw_diagnostics: Vec<RawDiagnostic>,
    ignored_nodes: HashSet<(usize, usize)>,
    /// Stack of ancestor nodes (parent chain).
    /// Stores the actual Node values for full access to parent node data.
    ancestors: Vec<Node<'rk>>,
}

impl<'rk> Checker<'rk> {
    pub fn new(source: &'rk [u8], config: &'rk Config) -> Self {
        Self {
            source,
            config,
            line_index: LineIndex::from_source(source),
            raw_diagnostics: Vec::new(),
            ignored_nodes: HashSet::new(),
            ancestors: Vec::with_capacity(32), // Pre-allocate for typical nesting depth
        }
    }

    pub fn source(&self) -> &[u8] {
        self.source
    }

    pub fn config(&self) -> &Config {
        self.config
    }

    pub fn line_index(&self) -> &LineIndex<'rk> {
        &self.line_index
    }

    // ========== Ancestor tracking ==========
    // NOTE: analyze is called BEFORE push_ancestor, so during analyze,
    // ancestors.last() is the parent (not the current node).

    /// Get the immediate parent node, if any.
    #[inline]
    pub fn parent(&self) -> Option<&Node<'rk>> {
        self.ancestors.last()
    }

    /// Get the Nth ancestor (0 = parent, 1 = grandparent, etc.)
    #[inline]
    pub fn ancestor(&self, n: usize) -> Option<&Node<'rk>> {
        let len = self.ancestors.len();
        let idx = n + 1; // 0 = parent = last, 1 = grandparent = len-2, etc.
        if idx <= len {
            Some(&self.ancestors[len - idx])
        } else {
            None
        }
    }

    /// Get all ancestors as a slice (from root to parent).
    #[inline]
    pub fn ancestors(&self) -> &[Node<'rk>] {
        &self.ancestors
    }

    /// Push a node onto the ancestor stack (called before visiting children).
    #[inline]
    fn push_ancestor(&mut self, node: Node<'_>) {
        // SAFETY: We know the node lifetime is valid during the visit traversal.
        // The ancestors Vec will be cleared before the source is invalidated.
        let node: Node<'rk> = unsafe { std::mem::transmute(node) };
        self.ancestors.push(node);
    }

    /// Pop the last ancestor from the stack (called after visiting children).
    #[inline]
    fn pop_ancestor(&mut self) {
        self.ancestors.pop();
    }

    // ========== Ignored nodes ==========

    /// Mark a node as ignored (will not be processed by rules).
    /// This is equivalent to RuboCop's `ignore_node`.
    #[inline]
    pub fn ignore_node(&mut self, location: &Location) {
        self.ignored_nodes
            .insert((location.start_offset(), location.end_offset()));
    }

    /// Check if a node is exactly one of the ignored nodes.
    /// This is equivalent to RuboCop's `ignored_node?`.
    #[inline]
    pub fn is_ignored_node(&self, start_offset: usize, end_offset: usize) -> bool {
        self.ignored_nodes.contains(&(start_offset, end_offset))
    }

    /// Check if a node is part of (contained within) any ignored node.
    /// This is equivalent to RuboCop's `part_of_ignored_node?`.
    #[inline]
    pub fn is_part_of_ignored_node(&self, start_offset: usize, end_offset: usize) -> bool {
        self.ignored_nodes
            .iter()
            .any(|&(ignored_start, ignored_end)| ignored_start <= start_offset && end_offset <= ignored_end)
    }

    /// Report a diagnostic (deferred line/column calculation).
    #[inline]
    pub fn report(
        &mut self,
        rule_id: RuleId,
        message: String,
        severity: Severity,
        start_offset: usize,
        end_offset: usize,
        fix: Option<Fix>,
    ) {
        self.raw_diagnostics.push(RawDiagnostic {
            rule_id,
            message,
            severity,
            start: start_offset,
            end: end_offset,
            fix,
        });
    }

    /// Convert raw diagnostics to full diagnostics with line/column info.
    /// Uses batch processing for efficient line number resolution.
    pub fn into_diagnostics(mut self) -> Vec<Diagnostic> {
        if self.raw_diagnostics.is_empty() {
            return Vec::new();
        }

        // Sort by offset for efficient batch resolution
        self.raw_diagnostics.sort_by_key(|d| (d.start, d.end));

        // Collect offsets for batch resolution
        let offsets: Vec<(usize, usize)> = self.raw_diagnostics.iter().map(|d| (d.start, d.end)).collect();

        // Batch resolve all line/column pairs
        let resolved = self.line_index.batch_line_column(&offsets);

        // Convert to full diagnostics
        self.raw_diagnostics
            .into_iter()
            .zip(resolved)
            .map(|(raw, (line_start, line_end, column_start, column_end))| {
                raw.resolve(line_start, line_end, column_start, column_end)
            })
            .collect()
    }
}

impl Visit<'_> for Checker<'_> {
    // Ancestor tracking: push current node on enter, pop on leave.
    // Since the current node is at the top of the stack, parent() returns ancestors[len-2].
    fn visit_branch_node_enter(&mut self, _node: ruby_prism::Node) {}
    fn visit_branch_node_leave(&mut self) {}
    fn visit_leaf_node_enter(&mut self, _node: ruby_prism::Node) {}
    fn visit_leaf_node_leave(&mut self) {}
    fn visit_alias_global_variable_node(&mut self, node: &ruby_prism::AliasGlobalVariableNode) {
        analyze::alias_global_variable_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_alias_global_variable_node(self, node);
        self.pop_ancestor();
    }
    fn visit_alias_method_node(&mut self, node: &ruby_prism::AliasMethodNode) {
        analyze::alias_method_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_alias_method_node(self, node);
        self.pop_ancestor();
    }
    fn visit_alternation_pattern_node(&mut self, node: &ruby_prism::AlternationPatternNode) {
        analyze::alternation_pattern_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_alternation_pattern_node(self, node);
        self.pop_ancestor();
    }
    fn visit_and_node(&mut self, node: &ruby_prism::AndNode) {
        analyze::and_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_and_node(self, node);
        self.pop_ancestor();
    }
    fn visit_arguments_node(&mut self, node: &ruby_prism::ArgumentsNode) {
        analyze::arguments_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_arguments_node(self, node);
        self.pop_ancestor();
    }
    fn visit_array_node(&mut self, node: &ruby_prism::ArrayNode) {
        analyze::array_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_array_node(self, node);
        self.pop_ancestor();
    }
    fn visit_array_pattern_node(&mut self, node: &ruby_prism::ArrayPatternNode) {
        analyze::array_pattern_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_array_pattern_node(self, node);
        self.pop_ancestor();
    }
    fn visit_assoc_node(&mut self, node: &ruby_prism::AssocNode) {
        analyze::assoc_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_assoc_node(self, node);
        self.pop_ancestor();
    }
    fn visit_assoc_splat_node(&mut self, node: &ruby_prism::AssocSplatNode) {
        analyze::assoc_splat_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_assoc_splat_node(self, node);
        self.pop_ancestor();
    }
    fn visit_back_reference_read_node(&mut self, node: &ruby_prism::BackReferenceReadNode) {
        analyze::back_reference_read_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_back_reference_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_begin_node(&mut self, node: &ruby_prism::BeginNode) {
        analyze::begin_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_begin_node(self, node);
        self.pop_ancestor();
    }
    fn visit_block_argument_node(&mut self, node: &ruby_prism::BlockArgumentNode) {
        analyze::block_argument_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_block_argument_node(self, node);
        self.pop_ancestor();
    }
    fn visit_block_local_variable_node(&mut self, node: &ruby_prism::BlockLocalVariableNode) {
        analyze::block_local_variable_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_block_local_variable_node(self, node);
        self.pop_ancestor();
    }
    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode) {
        analyze::block_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_block_node(self, node);
        self.pop_ancestor();
    }
    fn visit_block_parameter_node(&mut self, node: &ruby_prism::BlockParameterNode) {
        analyze::block_parameter_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_block_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_block_parameters_node(&mut self, node: &ruby_prism::BlockParametersNode) {
        analyze::block_parameters_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_block_parameters_node(self, node);
        self.pop_ancestor();
    }
    fn visit_break_node(&mut self, node: &ruby_prism::BreakNode) {
        analyze::break_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_break_node(self, node);
        self.pop_ancestor();
    }
    fn visit_call_and_write_node(&mut self, node: &ruby_prism::CallAndWriteNode) {
        analyze::call_and_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_call_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode) {
        analyze::call_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_call_node(self, node);
        self.pop_ancestor();
    }
    fn visit_call_operator_write_node(&mut self, node: &ruby_prism::CallOperatorWriteNode) {
        analyze::call_operator_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_call_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_call_or_write_node(&mut self, node: &ruby_prism::CallOrWriteNode) {
        analyze::call_or_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_call_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_call_target_node(&mut self, node: &ruby_prism::CallTargetNode) {
        analyze::call_target_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_call_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_capture_pattern_node(&mut self, node: &ruby_prism::CapturePatternNode) {
        analyze::capture_pattern_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_capture_pattern_node(self, node);
        self.pop_ancestor();
    }
    fn visit_case_match_node(&mut self, node: &ruby_prism::CaseMatchNode) {
        analyze::case_match_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_case_match_node(self, node);
        self.pop_ancestor();
    }
    fn visit_case_node(&mut self, node: &ruby_prism::CaseNode) {
        analyze::case_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_case_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_node(&mut self, node: &ruby_prism::ClassNode) {
        analyze::class_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_and_write_node(&mut self, node: &ruby_prism::ClassVariableAndWriteNode) {
        analyze::class_variable_and_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_operator_write_node(&mut self, node: &ruby_prism::ClassVariableOperatorWriteNode) {
        analyze::class_variable_operator_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_or_write_node(&mut self, node: &ruby_prism::ClassVariableOrWriteNode) {
        analyze::class_variable_or_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_read_node(&mut self, node: &ruby_prism::ClassVariableReadNode) {
        analyze::class_variable_read_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_target_node(&mut self, node: &ruby_prism::ClassVariableTargetNode) {
        analyze::class_variable_target_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_write_node(&mut self, node: &ruby_prism::ClassVariableWriteNode) {
        analyze::class_variable_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_and_write_node(&mut self, node: &ruby_prism::ConstantAndWriteNode) {
        analyze::constant_and_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_operator_write_node(&mut self, node: &ruby_prism::ConstantOperatorWriteNode) {
        analyze::constant_operator_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_or_write_node(&mut self, node: &ruby_prism::ConstantOrWriteNode) {
        analyze::constant_or_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_and_write_node(&mut self, node: &ruby_prism::ConstantPathAndWriteNode) {
        analyze::constant_path_and_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_node(&mut self, node: &ruby_prism::ConstantPathNode) {
        analyze::constant_path_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_operator_write_node(&mut self, node: &ruby_prism::ConstantPathOperatorWriteNode) {
        analyze::constant_path_operator_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_or_write_node(&mut self, node: &ruby_prism::ConstantPathOrWriteNode) {
        analyze::constant_path_or_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_target_node(&mut self, node: &ruby_prism::ConstantPathTargetNode) {
        analyze::constant_path_target_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_write_node(&mut self, node: &ruby_prism::ConstantPathWriteNode) {
        analyze::constant_path_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_read_node(&mut self, node: &ruby_prism::ConstantReadNode) {
        analyze::constant_read_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_target_node(&mut self, node: &ruby_prism::ConstantTargetNode) {
        analyze::constant_target_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_write_node(&mut self, node: &ruby_prism::ConstantWriteNode) {
        analyze::constant_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_def_node(&mut self, node: &ruby_prism::DefNode) {
        analyze::def_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_def_node(self, node);
        self.pop_ancestor();
    }
    fn visit_defined_node(&mut self, node: &ruby_prism::DefinedNode) {
        analyze::defined_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_defined_node(self, node);
        self.pop_ancestor();
    }
    fn visit_else_node(&mut self, node: &ruby_prism::ElseNode) {
        analyze::else_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_else_node(self, node);
        self.pop_ancestor();
    }
    fn visit_embedded_statements_node(&mut self, node: &ruby_prism::EmbeddedStatementsNode) {
        analyze::embedded_statements_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_embedded_statements_node(self, node);
        self.pop_ancestor();
    }
    fn visit_embedded_variable_node(&mut self, node: &ruby_prism::EmbeddedVariableNode) {
        analyze::embedded_variable_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_embedded_variable_node(self, node);
        self.pop_ancestor();
    }
    fn visit_ensure_node(&mut self, node: &ruby_prism::EnsureNode) {
        analyze::ensure_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_ensure_node(self, node);
        self.pop_ancestor();
    }
    fn visit_false_node(&mut self, node: &ruby_prism::FalseNode) {
        analyze::false_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_false_node(self, node);
        self.pop_ancestor();
    }
    fn visit_find_pattern_node(&mut self, node: &ruby_prism::FindPatternNode) {
        analyze::find_pattern_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_find_pattern_node(self, node);
        self.pop_ancestor();
    }
    fn visit_flip_flop_node(&mut self, node: &ruby_prism::FlipFlopNode) {
        analyze::flip_flop_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_flip_flop_node(self, node);
        self.pop_ancestor();
    }
    fn visit_float_node(&mut self, node: &ruby_prism::FloatNode) {
        analyze::float_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_float_node(self, node);
        self.pop_ancestor();
    }
    fn visit_for_node(&mut self, node: &ruby_prism::ForNode) {
        analyze::for_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_for_node(self, node);
        self.pop_ancestor();
    }
    fn visit_forwarding_arguments_node(&mut self, node: &ruby_prism::ForwardingArgumentsNode) {
        analyze::forwarding_arguments_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_forwarding_arguments_node(self, node);
        self.pop_ancestor();
    }
    fn visit_forwarding_parameter_node(&mut self, node: &ruby_prism::ForwardingParameterNode) {
        analyze::forwarding_parameter_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_forwarding_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_forwarding_super_node(&mut self, node: &ruby_prism::ForwardingSuperNode) {
        analyze::forwarding_super_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_forwarding_super_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_and_write_node(&mut self, node: &ruby_prism::GlobalVariableAndWriteNode) {
        analyze::global_variable_and_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_operator_write_node(&mut self, node: &ruby_prism::GlobalVariableOperatorWriteNode) {
        analyze::global_variable_operator_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_or_write_node(&mut self, node: &ruby_prism::GlobalVariableOrWriteNode) {
        analyze::global_variable_or_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_read_node(&mut self, node: &ruby_prism::GlobalVariableReadNode) {
        analyze::global_variable_read_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_target_node(&mut self, node: &ruby_prism::GlobalVariableTargetNode) {
        analyze::global_variable_target_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_write_node(&mut self, node: &ruby_prism::GlobalVariableWriteNode) {
        analyze::global_variable_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_hash_node(&mut self, node: &ruby_prism::HashNode) {
        analyze::hash_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_hash_node(self, node);
        self.pop_ancestor();
    }
    fn visit_hash_pattern_node(&mut self, node: &ruby_prism::HashPatternNode) {
        analyze::hash_pattern_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_hash_pattern_node(self, node);
        self.pop_ancestor();
    }
    fn visit_if_node(&mut self, node: &ruby_prism::IfNode) {
        analyze::if_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_if_node(self, node);
        self.pop_ancestor();
    }
    fn visit_imaginary_node(&mut self, node: &ruby_prism::ImaginaryNode) {
        analyze::imaginary_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_imaginary_node(self, node);
        self.pop_ancestor();
    }
    fn visit_implicit_node(&mut self, node: &ruby_prism::ImplicitNode) {
        analyze::implicit_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_implicit_node(self, node);
        self.pop_ancestor();
    }
    fn visit_implicit_rest_node(&mut self, node: &ruby_prism::ImplicitRestNode) {
        analyze::implicit_rest_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_implicit_rest_node(self, node);
        self.pop_ancestor();
    }
    fn visit_in_node(&mut self, node: &ruby_prism::InNode) {
        analyze::in_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_in_node(self, node);
        self.pop_ancestor();
    }
    fn visit_index_and_write_node(&mut self, node: &ruby_prism::IndexAndWriteNode) {
        analyze::index_and_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_index_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_index_operator_write_node(&mut self, node: &ruby_prism::IndexOperatorWriteNode) {
        analyze::index_operator_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_index_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_index_or_write_node(&mut self, node: &ruby_prism::IndexOrWriteNode) {
        analyze::index_or_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_index_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_index_target_node(&mut self, node: &ruby_prism::IndexTargetNode) {
        analyze::index_target_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_index_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_and_write_node(&mut self, node: &ruby_prism::InstanceVariableAndWriteNode) {
        analyze::instance_variable_and_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_operator_write_node(&mut self, node: &ruby_prism::InstanceVariableOperatorWriteNode) {
        analyze::instance_variable_operator_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_or_write_node(&mut self, node: &ruby_prism::InstanceVariableOrWriteNode) {
        analyze::instance_variable_or_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_read_node(&mut self, node: &ruby_prism::InstanceVariableReadNode) {
        analyze::instance_variable_read_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_target_node(&mut self, node: &ruby_prism::InstanceVariableTargetNode) {
        analyze::instance_variable_target_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_write_node(&mut self, node: &ruby_prism::InstanceVariableWriteNode) {
        analyze::instance_variable_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_integer_node(&mut self, node: &ruby_prism::IntegerNode) {
        analyze::integer_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_integer_node(self, node);
        self.pop_ancestor();
    }
    fn visit_interpolated_match_last_line_node(&mut self, node: &ruby_prism::InterpolatedMatchLastLineNode) {
        analyze::interpolated_match_last_line_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_interpolated_match_last_line_node(self, node);
        self.pop_ancestor();
    }
    fn visit_interpolated_regular_expression_node(&mut self, node: &ruby_prism::InterpolatedRegularExpressionNode) {
        analyze::interpolated_regular_expression_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_interpolated_regular_expression_node(self, node);
        self.pop_ancestor();
    }
    fn visit_interpolated_string_node(&mut self, node: &ruby_prism::InterpolatedStringNode) {
        analyze::interpolated_string_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_interpolated_string_node(self, node);
        self.pop_ancestor();
    }
    fn visit_interpolated_symbol_node(&mut self, node: &ruby_prism::InterpolatedSymbolNode) {
        analyze::interpolated_symbol_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_interpolated_symbol_node(self, node);
        self.pop_ancestor();
    }
    fn visit_interpolated_x_string_node(&mut self, node: &ruby_prism::InterpolatedXStringNode) {
        analyze::interpolated_x_string_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_interpolated_x_string_node(self, node);
        self.pop_ancestor();
    }
    fn visit_it_local_variable_read_node(&mut self, node: &ruby_prism::ItLocalVariableReadNode) {
        analyze::it_local_variable_read_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_it_local_variable_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_it_parameters_node(&mut self, node: &ruby_prism::ItParametersNode) {
        analyze::it_parameters_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_it_parameters_node(self, node);
        self.pop_ancestor();
    }
    fn visit_keyword_hash_node(&mut self, node: &ruby_prism::KeywordHashNode) {
        analyze::keyword_hash_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_keyword_hash_node(self, node);
        self.pop_ancestor();
    }
    fn visit_keyword_rest_parameter_node(&mut self, node: &ruby_prism::KeywordRestParameterNode) {
        analyze::keyword_rest_parameter_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_keyword_rest_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode) {
        analyze::lambda_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_lambda_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_and_write_node(&mut self, node: &ruby_prism::LocalVariableAndWriteNode) {
        analyze::local_variable_and_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_operator_write_node(&mut self, node: &ruby_prism::LocalVariableOperatorWriteNode) {
        analyze::local_variable_operator_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_or_write_node(&mut self, node: &ruby_prism::LocalVariableOrWriteNode) {
        analyze::local_variable_or_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode) {
        analyze::local_variable_read_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_target_node(&mut self, node: &ruby_prism::LocalVariableTargetNode) {
        analyze::local_variable_target_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode) {
        analyze::local_variable_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_match_last_line_node(&mut self, node: &ruby_prism::MatchLastLineNode) {
        analyze::match_last_line_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_match_last_line_node(self, node);
        self.pop_ancestor();
    }
    fn visit_match_predicate_node(&mut self, node: &ruby_prism::MatchPredicateNode) {
        analyze::match_predicate_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_match_predicate_node(self, node);
        self.pop_ancestor();
    }
    fn visit_match_required_node(&mut self, node: &ruby_prism::MatchRequiredNode) {
        analyze::match_required_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_match_required_node(self, node);
        self.pop_ancestor();
    }
    fn visit_match_write_node(&mut self, node: &ruby_prism::MatchWriteNode) {
        analyze::match_write_node(node, self);
        analyze::assignment(node.as_node(), node.call().as_node(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_match_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_missing_node(&mut self, node: &ruby_prism::MissingNode) {
        analyze::missing_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_missing_node(self, node);
        self.pop_ancestor();
    }
    fn visit_module_node(&mut self, node: &ruby_prism::ModuleNode) {
        analyze::module_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_module_node(self, node);
        self.pop_ancestor();
    }
    fn visit_multi_target_node(&mut self, node: &ruby_prism::MultiTargetNode) {
        analyze::multi_target_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_multi_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_multi_write_node(&mut self, node: &ruby_prism::MultiWriteNode) {
        analyze::multi_write_node(node, self);
        analyze::assignment(node.as_node(), node.value(), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_multi_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_next_node(&mut self, node: &ruby_prism::NextNode) {
        analyze::next_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_next_node(self, node);
        self.pop_ancestor();
    }
    fn visit_nil_node(&mut self, node: &ruby_prism::NilNode) {
        analyze::nil_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_nil_node(self, node);
        self.pop_ancestor();
    }
    fn visit_no_keywords_parameter_node(&mut self, node: &ruby_prism::NoKeywordsParameterNode) {
        analyze::no_keywords_parameter_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_no_keywords_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_numbered_parameters_node(&mut self, node: &ruby_prism::NumberedParametersNode) {
        analyze::numbered_parameters_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_numbered_parameters_node(self, node);
        self.pop_ancestor();
    }
    fn visit_numbered_reference_read_node(&mut self, node: &ruby_prism::NumberedReferenceReadNode) {
        analyze::numbered_reference_read_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_numbered_reference_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_optional_keyword_parameter_node(&mut self, node: &ruby_prism::OptionalKeywordParameterNode) {
        analyze::optional_keyword_parameter_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_optional_keyword_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_optional_parameter_node(&mut self, node: &ruby_prism::OptionalParameterNode) {
        analyze::optional_parameter_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_optional_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_or_node(&mut self, node: &ruby_prism::OrNode) {
        analyze::or_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_or_node(self, node);
        self.pop_ancestor();
    }
    fn visit_parameters_node(&mut self, node: &ruby_prism::ParametersNode) {
        analyze::parameters_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_parameters_node(self, node);
        self.pop_ancestor();
    }
    fn visit_parentheses_node(&mut self, node: &ruby_prism::ParenthesesNode) {
        analyze::parentheses_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_parentheses_node(self, node);
        self.pop_ancestor();
    }
    fn visit_pinned_expression_node(&mut self, node: &ruby_prism::PinnedExpressionNode) {
        analyze::pinned_expression_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_pinned_expression_node(self, node);
        self.pop_ancestor();
    }
    fn visit_pinned_variable_node(&mut self, node: &ruby_prism::PinnedVariableNode) {
        analyze::pinned_variable_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_pinned_variable_node(self, node);
        self.pop_ancestor();
    }
    fn visit_post_execution_node(&mut self, node: &ruby_prism::PostExecutionNode) {
        analyze::post_execution_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_post_execution_node(self, node);
        self.pop_ancestor();
    }
    fn visit_pre_execution_node(&mut self, node: &ruby_prism::PreExecutionNode) {
        analyze::pre_execution_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_pre_execution_node(self, node);
        self.pop_ancestor();
    }
    fn visit_program_node(&mut self, node: &ruby_prism::ProgramNode) {
        analyze::program_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_program_node(self, node);
        self.pop_ancestor();
    }
    fn visit_range_node(&mut self, node: &ruby_prism::RangeNode) {
        analyze::range_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_range_node(self, node);
        self.pop_ancestor();
    }
    fn visit_rational_node(&mut self, node: &ruby_prism::RationalNode) {
        analyze::rational_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_rational_node(self, node);
        self.pop_ancestor();
    }
    fn visit_redo_node(&mut self, node: &ruby_prism::RedoNode) {
        analyze::redo_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_redo_node(self, node);
        self.pop_ancestor();
    }
    fn visit_regular_expression_node(&mut self, node: &ruby_prism::RegularExpressionNode) {
        analyze::regular_expression_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_regular_expression_node(self, node);
        self.pop_ancestor();
    }
    fn visit_required_keyword_parameter_node(&mut self, node: &ruby_prism::RequiredKeywordParameterNode) {
        analyze::required_keyword_parameter_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_required_keyword_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_required_parameter_node(&mut self, node: &ruby_prism::RequiredParameterNode) {
        analyze::required_parameter_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_required_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_rescue_modifier_node(&mut self, node: &ruby_prism::RescueModifierNode) {
        analyze::rescue_modifier_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_rescue_modifier_node(self, node);
        self.pop_ancestor();
    }
    fn visit_rescue_node(&mut self, node: &ruby_prism::RescueNode) {
        analyze::rescue_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_rescue_node(self, node);
        self.pop_ancestor();
    }
    fn visit_rest_parameter_node(&mut self, node: &ruby_prism::RestParameterNode) {
        analyze::rest_parameter_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_rest_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_retry_node(&mut self, node: &ruby_prism::RetryNode) {
        analyze::retry_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_retry_node(self, node);
        self.pop_ancestor();
    }
    fn visit_return_node(&mut self, node: &ruby_prism::ReturnNode) {
        analyze::return_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_return_node(self, node);
        self.pop_ancestor();
    }
    fn visit_self_node(&mut self, node: &ruby_prism::SelfNode) {
        analyze::self_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_self_node(self, node);
        self.pop_ancestor();
    }
    fn visit_shareable_constant_node(&mut self, node: &ruby_prism::ShareableConstantNode) {
        analyze::shareable_constant_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_shareable_constant_node(self, node);
        self.pop_ancestor();
    }
    fn visit_singleton_class_node(&mut self, node: &ruby_prism::SingletonClassNode) {
        analyze::singleton_class_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_singleton_class_node(self, node);
        self.pop_ancestor();
    }
    fn visit_source_encoding_node(&mut self, node: &ruby_prism::SourceEncodingNode) {
        analyze::source_encoding_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_source_encoding_node(self, node);
        self.pop_ancestor();
    }
    fn visit_source_file_node(&mut self, node: &ruby_prism::SourceFileNode) {
        analyze::source_file_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_source_file_node(self, node);
        self.pop_ancestor();
    }
    fn visit_source_line_node(&mut self, node: &ruby_prism::SourceLineNode) {
        analyze::source_line_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_source_line_node(self, node);
        self.pop_ancestor();
    }
    fn visit_splat_node(&mut self, node: &ruby_prism::SplatNode) {
        analyze::splat_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_splat_node(self, node);
        self.pop_ancestor();
    }
    fn visit_statements_node(&mut self, node: &ruby_prism::StatementsNode) {
        analyze::statements_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_statements_node(self, node);
        self.pop_ancestor();
    }
    fn visit_string_node(&mut self, node: &ruby_prism::StringNode) {
        analyze::string_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_string_node(self, node);
        self.pop_ancestor();
    }
    fn visit_super_node(&mut self, node: &ruby_prism::SuperNode) {
        analyze::super_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_super_node(self, node);
        self.pop_ancestor();
    }
    fn visit_symbol_node(&mut self, node: &ruby_prism::SymbolNode) {
        analyze::symbol_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_symbol_node(self, node);
        self.pop_ancestor();
    }
    fn visit_true_node(&mut self, node: &ruby_prism::TrueNode) {
        analyze::true_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_true_node(self, node);
        self.pop_ancestor();
    }
    fn visit_undef_node(&mut self, node: &ruby_prism::UndefNode) {
        analyze::undef_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_undef_node(self, node);
        self.pop_ancestor();
    }
    fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode) {
        analyze::unless_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_unless_node(self, node);
        self.pop_ancestor();
    }
    fn visit_until_node(&mut self, node: &ruby_prism::UntilNode) {
        analyze::until_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_until_node(self, node);
        self.pop_ancestor();
    }
    fn visit_when_node(&mut self, node: &ruby_prism::WhenNode) {
        analyze::when_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_when_node(self, node);
        self.pop_ancestor();
    }
    fn visit_while_node(&mut self, node: &ruby_prism::WhileNode) {
        analyze::while_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_while_node(self, node);
        self.pop_ancestor();
    }
    fn visit_x_string_node(&mut self, node: &ruby_prism::XStringNode) {
        analyze::x_string_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_x_string_node(self, node);
        self.pop_ancestor();
    }
    fn visit_yield_node(&mut self, node: &ruby_prism::YieldNode) {
        analyze::yield_node(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_yield_node(self, node);
        self.pop_ancestor();
    }
}
