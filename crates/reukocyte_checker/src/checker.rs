use crate::config::Config;
use crate::diagnostic::RawDiagnostic;
use crate::locator::LineIndex;
use crate::rule::RuleId;
use crate::utility::assignment::AssignmentNode;
use crate::{Diagnostic, Fix, Severity};
use ruby_prism::{Location, Node, Visit};
use std::collections::HashSet;

// Include the auto-generated rule registry macros
include!(concat!(env!("OUT_DIR"), "/rule_registry.rs"));

/// The main checker that traverses the AST and runs rules.
pub struct Checker<'rk> {
    source: &'rk [u8],
    config: &'rk Config,
    line_index: LineIndex<'rk>,
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
        if idx <= len { Some(&self.ancestors[len - idx]) } else { None }
    }

    /// Get all ancestors as a slice (from root to parent).
    #[inline]
    pub fn ancestors(&self) -> &[Node<'rk>] {
        &self.ancestors
    }

    /// Check if any ancestor matches the given predicate.
    ///
    /// # Example
    /// ```ignore
    /// // Check if we're inside a class definition
    /// if checker.has_ancestor(|node| node.as_class_node().is_some()) {
    ///     // ...
    /// }
    /// ```
    #[inline]
    pub fn has_ancestor<F>(&self, predicate: F) -> bool
    where
        F: Fn(&Node<'rk>) -> bool,
    {
        self.ancestors.iter().any(predicate)
    }

    /// Find the first ancestor that matches the given predicate (closest to current node).
    ///
    /// # Example
    /// ```ignore
    /// // Find the enclosing class node
    /// if let Some(class_node) = checker.find_ancestor(|node| node.as_class_node().is_some()) {
    ///     // ...
    /// }
    /// ```
    #[inline]
    pub fn find_ancestor<F>(&self, predicate: F) -> Option<&Node<'rk>>
    where
        F: Fn(&Node<'rk>) -> bool,
    {
        self.ancestors.iter().rev().find(|node| predicate(node))
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
        self.ignored_nodes.insert((location.start_offset(), location.end_offset()));
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
    pub fn report(&mut self, rule_id: RuleId, message: String, severity: Severity, start_offset: usize, end_offset: usize, fix: Option<Fix>) {
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
            .map(|(raw, (line_start, line_end, column_start, column_end))| raw.resolve(line_start, line_end, column_start, column_end))
            .collect()
    }

    // ========== Rule enablement ==========

    /// Check if a rule is enabled.
    ///
    /// TODO: Implement rule enable/disable logic based on config.
    /// For now, all rules are enabled.
    #[inline]
    pub fn is_enabled(&self, _rule_id: RuleId) -> bool {
        true
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
        run_alias_global_variable_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_alias_global_variable_node(self, node);
        self.pop_ancestor();
    }
    fn visit_alias_method_node(&mut self, node: &ruby_prism::AliasMethodNode) {
        run_alias_method_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_alias_method_node(self, node);
        self.pop_ancestor();
    }
    fn visit_alternation_pattern_node(&mut self, node: &ruby_prism::AlternationPatternNode) {
        run_alternation_pattern_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_alternation_pattern_node(self, node);
        self.pop_ancestor();
    }
    fn visit_and_node(&mut self, node: &ruby_prism::AndNode) {
        run_and_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_and_node(self, node);
        self.pop_ancestor();
    }
    fn visit_arguments_node(&mut self, node: &ruby_prism::ArgumentsNode) {
        run_arguments_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_arguments_node(self, node);
        self.pop_ancestor();
    }
    fn visit_array_node(&mut self, node: &ruby_prism::ArrayNode) {
        run_array_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_array_node(self, node);
        self.pop_ancestor();
    }
    fn visit_array_pattern_node(&mut self, node: &ruby_prism::ArrayPatternNode) {
        run_array_pattern_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_array_pattern_node(self, node);
        self.pop_ancestor();
    }
    fn visit_assoc_node(&mut self, node: &ruby_prism::AssocNode) {
        run_assoc_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_assoc_node(self, node);
        self.pop_ancestor();
    }
    fn visit_assoc_splat_node(&mut self, node: &ruby_prism::AssocSplatNode) {
        run_assoc_splat_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_assoc_splat_node(self, node);
        self.pop_ancestor();
    }
    fn visit_back_reference_read_node(&mut self, node: &ruby_prism::BackReferenceReadNode) {
        run_back_reference_read_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_back_reference_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_begin_node(&mut self, node: &ruby_prism::BeginNode) {
        run_begin_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_begin_node(self, node);
        self.pop_ancestor();
    }
    fn visit_block_argument_node(&mut self, node: &ruby_prism::BlockArgumentNode) {
        run_block_argument_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_block_argument_node(self, node);
        self.pop_ancestor();
    }
    fn visit_block_local_variable_node(&mut self, node: &ruby_prism::BlockLocalVariableNode) {
        run_block_local_variable_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_block_local_variable_node(self, node);
        self.pop_ancestor();
    }
    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode) {
        run_block_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_block_node(self, node);
        self.pop_ancestor();
    }
    fn visit_block_parameter_node(&mut self, node: &ruby_prism::BlockParameterNode) {
        run_block_parameter_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_block_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_block_parameters_node(&mut self, node: &ruby_prism::BlockParametersNode) {
        run_block_parameters_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_block_parameters_node(self, node);
        self.pop_ancestor();
    }
    fn visit_break_node(&mut self, node: &ruby_prism::BreakNode) {
        run_break_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_break_node(self, node);
        self.pop_ancestor();
    }
    fn visit_call_and_write_node(&mut self, node: &ruby_prism::CallAndWriteNode) {
        run_call_and_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_call_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode) {
        run_call_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_call_node(self, node);
        self.pop_ancestor();
    }
    fn visit_call_operator_write_node(&mut self, node: &ruby_prism::CallOperatorWriteNode) {
        run_call_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_call_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_call_or_write_node(&mut self, node: &ruby_prism::CallOrWriteNode) {
        run_call_or_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_call_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_call_target_node(&mut self, node: &ruby_prism::CallTargetNode) {
        run_call_target_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_call_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_capture_pattern_node(&mut self, node: &ruby_prism::CapturePatternNode) {
        run_capture_pattern_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_capture_pattern_node(self, node);
        self.pop_ancestor();
    }
    fn visit_case_match_node(&mut self, node: &ruby_prism::CaseMatchNode) {
        run_case_match_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_case_match_node(self, node);
        self.pop_ancestor();
    }
    fn visit_case_node(&mut self, node: &ruby_prism::CaseNode) {
        run_case_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_case_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_node(&mut self, node: &ruby_prism::ClassNode) {
        run_class_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_and_write_node(&mut self, node: &ruby_prism::ClassVariableAndWriteNode) {
        run_class_variable_and_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_operator_write_node(&mut self, node: &ruby_prism::ClassVariableOperatorWriteNode) {
        run_class_variable_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_or_write_node(&mut self, node: &ruby_prism::ClassVariableOrWriteNode) {
        run_class_variable_or_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_read_node(&mut self, node: &ruby_prism::ClassVariableReadNode) {
        run_class_variable_read_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_target_node(&mut self, node: &ruby_prism::ClassVariableTargetNode) {
        run_class_variable_target_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_write_node(&mut self, node: &ruby_prism::ClassVariableWriteNode) {
        run_class_variable_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_and_write_node(&mut self, node: &ruby_prism::ConstantAndWriteNode) {
        run_constant_and_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_operator_write_node(&mut self, node: &ruby_prism::ConstantOperatorWriteNode) {
        run_constant_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_or_write_node(&mut self, node: &ruby_prism::ConstantOrWriteNode) {
        run_constant_or_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_and_write_node(&mut self, node: &ruby_prism::ConstantPathAndWriteNode) {
        run_constant_path_and_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_node(&mut self, node: &ruby_prism::ConstantPathNode) {
        run_constant_path_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_operator_write_node(&mut self, node: &ruby_prism::ConstantPathOperatorWriteNode) {
        run_constant_path_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_or_write_node(&mut self, node: &ruby_prism::ConstantPathOrWriteNode) {
        run_constant_path_or_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_target_node(&mut self, node: &ruby_prism::ConstantPathTargetNode) {
        run_constant_path_target_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_write_node(&mut self, node: &ruby_prism::ConstantPathWriteNode) {
        run_constant_path_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_read_node(&mut self, node: &ruby_prism::ConstantReadNode) {
        run_constant_read_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_target_node(&mut self, node: &ruby_prism::ConstantTargetNode) {
        run_constant_target_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_write_node(&mut self, node: &ruby_prism::ConstantWriteNode) {
        run_constant_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_def_node(&mut self, node: &ruby_prism::DefNode) {
        run_def_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_def_node(self, node);
        self.pop_ancestor();
    }
    fn visit_defined_node(&mut self, node: &ruby_prism::DefinedNode) {
        run_defined_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_defined_node(self, node);
        self.pop_ancestor();
    }
    fn visit_else_node(&mut self, node: &ruby_prism::ElseNode) {
        run_else_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_else_node(self, node);
        self.pop_ancestor();
    }
    fn visit_embedded_statements_node(&mut self, node: &ruby_prism::EmbeddedStatementsNode) {
        run_embedded_statements_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_embedded_statements_node(self, node);
        self.pop_ancestor();
    }
    fn visit_embedded_variable_node(&mut self, node: &ruby_prism::EmbeddedVariableNode) {
        run_embedded_variable_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_embedded_variable_node(self, node);
        self.pop_ancestor();
    }
    fn visit_ensure_node(&mut self, node: &ruby_prism::EnsureNode) {
        run_ensure_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_ensure_node(self, node);
        self.pop_ancestor();
    }
    fn visit_false_node(&mut self, node: &ruby_prism::FalseNode) {
        run_false_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_false_node(self, node);
        self.pop_ancestor();
    }
    fn visit_find_pattern_node(&mut self, node: &ruby_prism::FindPatternNode) {
        run_find_pattern_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_find_pattern_node(self, node);
        self.pop_ancestor();
    }
    fn visit_flip_flop_node(&mut self, node: &ruby_prism::FlipFlopNode) {
        run_flip_flop_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_flip_flop_node(self, node);
        self.pop_ancestor();
    }
    fn visit_float_node(&mut self, node: &ruby_prism::FloatNode) {
        run_float_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_float_node(self, node);
        self.pop_ancestor();
    }
    fn visit_for_node(&mut self, node: &ruby_prism::ForNode) {
        run_for_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_for_node(self, node);
        self.pop_ancestor();
    }
    fn visit_forwarding_arguments_node(&mut self, node: &ruby_prism::ForwardingArgumentsNode) {
        run_forwarding_arguments_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_forwarding_arguments_node(self, node);
        self.pop_ancestor();
    }
    fn visit_forwarding_parameter_node(&mut self, node: &ruby_prism::ForwardingParameterNode) {
        run_forwarding_parameter_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_forwarding_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_forwarding_super_node(&mut self, node: &ruby_prism::ForwardingSuperNode) {
        run_forwarding_super_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_forwarding_super_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_and_write_node(&mut self, node: &ruby_prism::GlobalVariableAndWriteNode) {
        run_global_variable_and_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_operator_write_node(&mut self, node: &ruby_prism::GlobalVariableOperatorWriteNode) {
        run_global_variable_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_or_write_node(&mut self, node: &ruby_prism::GlobalVariableOrWriteNode) {
        run_global_variable_or_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_read_node(&mut self, node: &ruby_prism::GlobalVariableReadNode) {
        run_global_variable_read_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_target_node(&mut self, node: &ruby_prism::GlobalVariableTargetNode) {
        run_global_variable_target_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_write_node(&mut self, node: &ruby_prism::GlobalVariableWriteNode) {
        run_global_variable_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_hash_node(&mut self, node: &ruby_prism::HashNode) {
        run_hash_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_hash_node(self, node);
        self.pop_ancestor();
    }
    fn visit_hash_pattern_node(&mut self, node: &ruby_prism::HashPatternNode) {
        run_hash_pattern_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_hash_pattern_node(self, node);
        self.pop_ancestor();
    }
    fn visit_if_node(&mut self, node: &ruby_prism::IfNode) {
        run_if_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_if_node(self, node);
        self.pop_ancestor();
    }
    fn visit_imaginary_node(&mut self, node: &ruby_prism::ImaginaryNode) {
        run_imaginary_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_imaginary_node(self, node);
        self.pop_ancestor();
    }
    fn visit_implicit_node(&mut self, node: &ruby_prism::ImplicitNode) {
        run_implicit_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_implicit_node(self, node);
        self.pop_ancestor();
    }
    fn visit_implicit_rest_node(&mut self, node: &ruby_prism::ImplicitRestNode) {
        run_implicit_rest_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_implicit_rest_node(self, node);
        self.pop_ancestor();
    }
    fn visit_in_node(&mut self, node: &ruby_prism::InNode) {
        run_in_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_in_node(self, node);
        self.pop_ancestor();
    }
    fn visit_index_and_write_node(&mut self, node: &ruby_prism::IndexAndWriteNode) {
        run_index_and_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_index_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_index_operator_write_node(&mut self, node: &ruby_prism::IndexOperatorWriteNode) {
        run_index_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_index_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_index_or_write_node(&mut self, node: &ruby_prism::IndexOrWriteNode) {
        run_index_or_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_index_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_index_target_node(&mut self, node: &ruby_prism::IndexTargetNode) {
        run_index_target_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_index_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_and_write_node(&mut self, node: &ruby_prism::InstanceVariableAndWriteNode) {
        run_instance_variable_and_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_operator_write_node(&mut self, node: &ruby_prism::InstanceVariableOperatorWriteNode) {
        run_instance_variable_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_or_write_node(&mut self, node: &ruby_prism::InstanceVariableOrWriteNode) {
        run_instance_variable_or_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_read_node(&mut self, node: &ruby_prism::InstanceVariableReadNode) {
        run_instance_variable_read_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_target_node(&mut self, node: &ruby_prism::InstanceVariableTargetNode) {
        run_instance_variable_target_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_write_node(&mut self, node: &ruby_prism::InstanceVariableWriteNode) {
        run_instance_variable_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_integer_node(&mut self, node: &ruby_prism::IntegerNode) {
        run_integer_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_integer_node(self, node);
        self.pop_ancestor();
    }
    fn visit_interpolated_match_last_line_node(&mut self, node: &ruby_prism::InterpolatedMatchLastLineNode) {
        run_interpolated_match_last_line_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_interpolated_match_last_line_node(self, node);
        self.pop_ancestor();
    }
    fn visit_interpolated_regular_expression_node(&mut self, node: &ruby_prism::InterpolatedRegularExpressionNode) {
        run_interpolated_regular_expression_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_interpolated_regular_expression_node(self, node);
        self.pop_ancestor();
    }
    fn visit_interpolated_string_node(&mut self, node: &ruby_prism::InterpolatedStringNode) {
        run_interpolated_string_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_interpolated_string_node(self, node);
        self.pop_ancestor();
    }
    fn visit_interpolated_symbol_node(&mut self, node: &ruby_prism::InterpolatedSymbolNode) {
        run_interpolated_symbol_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_interpolated_symbol_node(self, node);
        self.pop_ancestor();
    }
    fn visit_interpolated_x_string_node(&mut self, node: &ruby_prism::InterpolatedXStringNode) {
        run_interpolated_x_string_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_interpolated_x_string_node(self, node);
        self.pop_ancestor();
    }
    fn visit_it_local_variable_read_node(&mut self, node: &ruby_prism::ItLocalVariableReadNode) {
        run_it_local_variable_read_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_it_local_variable_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_it_parameters_node(&mut self, node: &ruby_prism::ItParametersNode) {
        run_it_parameters_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_it_parameters_node(self, node);
        self.pop_ancestor();
    }
    fn visit_keyword_hash_node(&mut self, node: &ruby_prism::KeywordHashNode) {
        run_keyword_hash_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_keyword_hash_node(self, node);
        self.pop_ancestor();
    }
    fn visit_keyword_rest_parameter_node(&mut self, node: &ruby_prism::KeywordRestParameterNode) {
        run_keyword_rest_parameter_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_keyword_rest_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode) {
        run_lambda_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_lambda_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_and_write_node(&mut self, node: &ruby_prism::LocalVariableAndWriteNode) {
        run_local_variable_and_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_operator_write_node(&mut self, node: &ruby_prism::LocalVariableOperatorWriteNode) {
        run_local_variable_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_or_write_node(&mut self, node: &ruby_prism::LocalVariableOrWriteNode) {
        run_local_variable_or_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode) {
        run_local_variable_read_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_target_node(&mut self, node: &ruby_prism::LocalVariableTargetNode) {
        run_local_variable_target_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode) {
        run_local_variable_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_match_last_line_node(&mut self, node: &ruby_prism::MatchLastLineNode) {
        run_match_last_line_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_match_last_line_node(self, node);
        self.pop_ancestor();
    }
    fn visit_match_predicate_node(&mut self, node: &ruby_prism::MatchPredicateNode) {
        run_match_predicate_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_match_predicate_node(self, node);
        self.pop_ancestor();
    }
    fn visit_match_required_node(&mut self, node: &ruby_prism::MatchRequiredNode) {
        run_match_required_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_match_required_node(self, node);
        self.pop_ancestor();
    }
    fn visit_match_write_node(&mut self, node: &ruby_prism::MatchWriteNode) {
        run_match_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_match_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_missing_node(&mut self, node: &ruby_prism::MissingNode) {
        run_missing_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_missing_node(self, node);
        self.pop_ancestor();
    }
    fn visit_module_node(&mut self, node: &ruby_prism::ModuleNode) {
        run_module_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_module_node(self, node);
        self.pop_ancestor();
    }
    fn visit_multi_target_node(&mut self, node: &ruby_prism::MultiTargetNode) {
        run_multi_target_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_multi_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_multi_write_node(&mut self, node: &ruby_prism::MultiWriteNode) {
        run_multi_write_node_rules!(node, self);
        run_assignment_node_rules!(AssignmentNode::from(node), self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_multi_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_next_node(&mut self, node: &ruby_prism::NextNode) {
        run_next_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_next_node(self, node);
        self.pop_ancestor();
    }
    fn visit_nil_node(&mut self, node: &ruby_prism::NilNode) {
        run_nil_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_nil_node(self, node);
        self.pop_ancestor();
    }
    fn visit_no_keywords_parameter_node(&mut self, node: &ruby_prism::NoKeywordsParameterNode) {
        run_no_keywords_parameter_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_no_keywords_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_numbered_parameters_node(&mut self, node: &ruby_prism::NumberedParametersNode) {
        run_numbered_parameters_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_numbered_parameters_node(self, node);
        self.pop_ancestor();
    }
    fn visit_numbered_reference_read_node(&mut self, node: &ruby_prism::NumberedReferenceReadNode) {
        run_numbered_reference_read_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_numbered_reference_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_optional_keyword_parameter_node(&mut self, node: &ruby_prism::OptionalKeywordParameterNode) {
        run_optional_keyword_parameter_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_optional_keyword_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_optional_parameter_node(&mut self, node: &ruby_prism::OptionalParameterNode) {
        run_optional_parameter_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_optional_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_or_node(&mut self, node: &ruby_prism::OrNode) {
        run_or_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_or_node(self, node);
        self.pop_ancestor();
    }
    fn visit_parameters_node(&mut self, node: &ruby_prism::ParametersNode) {
        run_parameters_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_parameters_node(self, node);
        self.pop_ancestor();
    }
    fn visit_parentheses_node(&mut self, node: &ruby_prism::ParenthesesNode) {
        run_parentheses_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_parentheses_node(self, node);
        self.pop_ancestor();
    }
    fn visit_pinned_expression_node(&mut self, node: &ruby_prism::PinnedExpressionNode) {
        run_pinned_expression_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_pinned_expression_node(self, node);
        self.pop_ancestor();
    }
    fn visit_pinned_variable_node(&mut self, node: &ruby_prism::PinnedVariableNode) {
        run_pinned_variable_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_pinned_variable_node(self, node);
        self.pop_ancestor();
    }
    fn visit_post_execution_node(&mut self, node: &ruby_prism::PostExecutionNode) {
        run_post_execution_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_post_execution_node(self, node);
        self.pop_ancestor();
    }
    fn visit_pre_execution_node(&mut self, node: &ruby_prism::PreExecutionNode) {
        run_pre_execution_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_pre_execution_node(self, node);
        self.pop_ancestor();
    }
    fn visit_program_node(&mut self, node: &ruby_prism::ProgramNode) {
        run_program_node_rules!(node, self);
        // self.push_ancestor(node.as_node());
        ruby_prism::visit_program_node(self, node);
        // self.pop_ancestor();
    }
    fn visit_range_node(&mut self, node: &ruby_prism::RangeNode) {
        run_range_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_range_node(self, node);
        self.pop_ancestor();
    }
    fn visit_rational_node(&mut self, node: &ruby_prism::RationalNode) {
        run_rational_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_rational_node(self, node);
        self.pop_ancestor();
    }
    fn visit_redo_node(&mut self, node: &ruby_prism::RedoNode) {
        run_redo_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_redo_node(self, node);
        self.pop_ancestor();
    }
    fn visit_regular_expression_node(&mut self, node: &ruby_prism::RegularExpressionNode) {
        run_regular_expression_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_regular_expression_node(self, node);
        self.pop_ancestor();
    }
    fn visit_required_keyword_parameter_node(&mut self, node: &ruby_prism::RequiredKeywordParameterNode) {
        run_required_keyword_parameter_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_required_keyword_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_required_parameter_node(&mut self, node: &ruby_prism::RequiredParameterNode) {
        run_required_parameter_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_required_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_rescue_modifier_node(&mut self, node: &ruby_prism::RescueModifierNode) {
        run_rescue_modifier_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_rescue_modifier_node(self, node);
        self.pop_ancestor();
    }
    fn visit_rescue_node(&mut self, node: &ruby_prism::RescueNode) {
        run_rescue_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_rescue_node(self, node);
        self.pop_ancestor();
    }
    fn visit_rest_parameter_node(&mut self, node: &ruby_prism::RestParameterNode) {
        run_rest_parameter_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_rest_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_retry_node(&mut self, node: &ruby_prism::RetryNode) {
        run_retry_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_retry_node(self, node);
        self.pop_ancestor();
    }
    fn visit_return_node(&mut self, node: &ruby_prism::ReturnNode) {
        run_return_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_return_node(self, node);
        self.pop_ancestor();
    }
    fn visit_self_node(&mut self, node: &ruby_prism::SelfNode) {
        run_self_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_self_node(self, node);
        self.pop_ancestor();
    }
    fn visit_shareable_constant_node(&mut self, node: &ruby_prism::ShareableConstantNode) {
        run_shareable_constant_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_shareable_constant_node(self, node);
        self.pop_ancestor();
    }
    fn visit_singleton_class_node(&mut self, node: &ruby_prism::SingletonClassNode) {
        run_singleton_class_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_singleton_class_node(self, node);
        self.pop_ancestor();
    }
    fn visit_source_encoding_node(&mut self, node: &ruby_prism::SourceEncodingNode) {
        run_source_encoding_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_source_encoding_node(self, node);
        self.pop_ancestor();
    }
    fn visit_source_file_node(&mut self, node: &ruby_prism::SourceFileNode) {
        run_source_file_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_source_file_node(self, node);
        self.pop_ancestor();
    }
    fn visit_source_line_node(&mut self, node: &ruby_prism::SourceLineNode) {
        run_source_line_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_source_line_node(self, node);
        self.pop_ancestor();
    }
    fn visit_splat_node(&mut self, node: &ruby_prism::SplatNode) {
        run_splat_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_splat_node(self, node);
        self.pop_ancestor();
    }
    fn visit_statements_node(&mut self, node: &ruby_prism::StatementsNode) {
        run_statements_node_rules!(node, self);
        // Skip adding ProgramNode's StatementsNode to ancestors
        // (ProgramNode is the root and not pushed to ancestors)
        let should_track = self.parent().is_some();
        should_track.then(|| self.push_ancestor(node.as_node()));
        ruby_prism::visit_statements_node(self, node);
        should_track.then(|| self.pop_ancestor());
    }
    fn visit_string_node(&mut self, node: &ruby_prism::StringNode) {
        run_string_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_string_node(self, node);
        self.pop_ancestor();
    }
    fn visit_super_node(&mut self, node: &ruby_prism::SuperNode) {
        run_super_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_super_node(self, node);
        self.pop_ancestor();
    }
    fn visit_symbol_node(&mut self, node: &ruby_prism::SymbolNode) {
        run_symbol_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_symbol_node(self, node);
        self.pop_ancestor();
    }
    fn visit_true_node(&mut self, node: &ruby_prism::TrueNode) {
        run_true_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_true_node(self, node);
        self.pop_ancestor();
    }
    fn visit_undef_node(&mut self, node: &ruby_prism::UndefNode) {
        run_undef_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_undef_node(self, node);
        self.pop_ancestor();
    }
    fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode) {
        run_unless_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_unless_node(self, node);
        self.pop_ancestor();
    }
    fn visit_until_node(&mut self, node: &ruby_prism::UntilNode) {
        run_until_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_until_node(self, node);
        self.pop_ancestor();
    }
    fn visit_when_node(&mut self, node: &ruby_prism::WhenNode) {
        run_when_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_when_node(self, node);
        self.pop_ancestor();
    }
    fn visit_while_node(&mut self, node: &ruby_prism::WhileNode) {
        run_while_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_while_node(self, node);
        self.pop_ancestor();
    }
    fn visit_x_string_node(&mut self, node: &ruby_prism::XStringNode) {
        run_x_string_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_x_string_node(self, node);
        self.pop_ancestor();
    }
    fn visit_yield_node(&mut self, node: &ruby_prism::YieldNode) {
        run_yield_node_rules!(node, self);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_yield_node(self, node);
        self.pop_ancestor();
    }
}
