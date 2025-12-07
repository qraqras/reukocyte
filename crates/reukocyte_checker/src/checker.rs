use crate::config::Config;
use crate::diagnostic::RawDiagnostic;
use crate::locator::LineIndex;
use crate::rule::RuleId;
use crate::run_rules;
use crate::utility::assignment::AssignmentNode;
use crate::{Diagnostic, Fix, Severity};
use ruby_prism::{Location, Node, Visit};
use std::collections::HashSet;

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
        run_rules!(node, self, AliasGlobalVariableNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_alias_global_variable_node(self, node);
        self.pop_ancestor();
    }
    fn visit_alias_method_node(&mut self, node: &ruby_prism::AliasMethodNode) {
        run_rules!(node, self, AliasMethodNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_alias_method_node(self, node);
        self.pop_ancestor();
    }
    fn visit_alternation_pattern_node(&mut self, node: &ruby_prism::AlternationPatternNode) {
        run_rules!(node, self, AlternationPatternNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_alternation_pattern_node(self, node);
        self.pop_ancestor();
    }
    fn visit_and_node(&mut self, node: &ruby_prism::AndNode) {
        run_rules!(node, self, AndNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_and_node(self, node);
        self.pop_ancestor();
    }
    fn visit_arguments_node(&mut self, node: &ruby_prism::ArgumentsNode) {
        run_rules!(node, self, ArgumentsNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_arguments_node(self, node);
        self.pop_ancestor();
    }
    fn visit_array_node(&mut self, node: &ruby_prism::ArrayNode) {
        run_rules!(node, self, ArrayNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_array_node(self, node);
        self.pop_ancestor();
    }
    fn visit_array_pattern_node(&mut self, node: &ruby_prism::ArrayPatternNode) {
        run_rules!(node, self, ArrayPatternNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_array_pattern_node(self, node);
        self.pop_ancestor();
    }
    fn visit_assoc_node(&mut self, node: &ruby_prism::AssocNode) {
        run_rules!(node, self, AssocNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_assoc_node(self, node);
        self.pop_ancestor();
    }
    fn visit_assoc_splat_node(&mut self, node: &ruby_prism::AssocSplatNode) {
        run_rules!(node, self, AssocSplatNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_assoc_splat_node(self, node);
        self.pop_ancestor();
    }
    fn visit_back_reference_read_node(&mut self, node: &ruby_prism::BackReferenceReadNode) {
        run_rules!(node, self, BackReferenceReadNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_back_reference_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_begin_node(&mut self, node: &ruby_prism::BeginNode) {
        run_rules!(node, self, BeginNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_begin_node(self, node);
        self.pop_ancestor();
    }
    fn visit_block_argument_node(&mut self, node: &ruby_prism::BlockArgumentNode) {
        run_rules!(node, self, BlockArgumentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_block_argument_node(self, node);
        self.pop_ancestor();
    }
    fn visit_block_local_variable_node(&mut self, node: &ruby_prism::BlockLocalVariableNode) {
        run_rules!(node, self, BlockLocalVariableNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_block_local_variable_node(self, node);
        self.pop_ancestor();
    }
    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode) {
        run_rules!(node, self, BlockNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_block_node(self, node);
        self.pop_ancestor();
    }
    fn visit_block_parameter_node(&mut self, node: &ruby_prism::BlockParameterNode) {
        run_rules!(node, self, BlockParameterNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_block_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_block_parameters_node(&mut self, node: &ruby_prism::BlockParametersNode) {
        run_rules!(node, self, BlockParametersNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_block_parameters_node(self, node);
        self.pop_ancestor();
    }
    fn visit_break_node(&mut self, node: &ruby_prism::BreakNode) {
        run_rules!(node, self, BreakNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_break_node(self, node);
        self.pop_ancestor();
    }
    fn visit_call_and_write_node(&mut self, node: &ruby_prism::CallAndWriteNode) {
        run_rules!(node, self, CallAndWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_call_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode) {
        run_rules!(node, self, CallNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_call_node(self, node);
        self.pop_ancestor();
    }
    fn visit_call_operator_write_node(&mut self, node: &ruby_prism::CallOperatorWriteNode) {
        run_rules!(node, self, CallOperatorWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_call_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_call_or_write_node(&mut self, node: &ruby_prism::CallOrWriteNode) {
        run_rules!(node, self, CallOrWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_call_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_call_target_node(&mut self, node: &ruby_prism::CallTargetNode) {
        run_rules!(node, self, CallTargetNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_call_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_capture_pattern_node(&mut self, node: &ruby_prism::CapturePatternNode) {
        run_rules!(node, self, CapturePatternNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_capture_pattern_node(self, node);
        self.pop_ancestor();
    }
    fn visit_case_match_node(&mut self, node: &ruby_prism::CaseMatchNode) {
        run_rules!(node, self, CaseMatchNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_case_match_node(self, node);
        self.pop_ancestor();
    }
    fn visit_case_node(&mut self, node: &ruby_prism::CaseNode) {
        run_rules!(node, self, CaseNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_case_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_node(&mut self, node: &ruby_prism::ClassNode) {
        run_rules!(node, self, ClassNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_and_write_node(&mut self, node: &ruby_prism::ClassVariableAndWriteNode) {
        run_rules!(node, self, ClassVariableAndWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_operator_write_node(&mut self, node: &ruby_prism::ClassVariableOperatorWriteNode) {
        run_rules!(node, self, ClassVariableOperatorWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_or_write_node(&mut self, node: &ruby_prism::ClassVariableOrWriteNode) {
        run_rules!(node, self, ClassVariableOrWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_read_node(&mut self, node: &ruby_prism::ClassVariableReadNode) {
        run_rules!(node, self, ClassVariableReadNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_target_node(&mut self, node: &ruby_prism::ClassVariableTargetNode) {
        run_rules!(node, self, ClassVariableTargetNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_class_variable_write_node(&mut self, node: &ruby_prism::ClassVariableWriteNode) {
        run_rules!(node, self, ClassVariableWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_class_variable_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_and_write_node(&mut self, node: &ruby_prism::ConstantAndWriteNode) {
        run_rules!(node, self, ConstantAndWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_operator_write_node(&mut self, node: &ruby_prism::ConstantOperatorWriteNode) {
        run_rules!(node, self, ConstantOperatorWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_or_write_node(&mut self, node: &ruby_prism::ConstantOrWriteNode) {
        run_rules!(node, self, ConstantOrWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_and_write_node(&mut self, node: &ruby_prism::ConstantPathAndWriteNode) {
        run_rules!(node, self, ConstantPathAndWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_node(&mut self, node: &ruby_prism::ConstantPathNode) {
        run_rules!(node, self, ConstantPathNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_operator_write_node(&mut self, node: &ruby_prism::ConstantPathOperatorWriteNode) {
        run_rules!(node, self, ConstantPathOperatorWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_or_write_node(&mut self, node: &ruby_prism::ConstantPathOrWriteNode) {
        run_rules!(node, self, ConstantPathOrWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_target_node(&mut self, node: &ruby_prism::ConstantPathTargetNode) {
        run_rules!(node, self, ConstantPathTargetNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_path_write_node(&mut self, node: &ruby_prism::ConstantPathWriteNode) {
        run_rules!(node, self, ConstantPathWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_path_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_read_node(&mut self, node: &ruby_prism::ConstantReadNode) {
        run_rules!(node, self, ConstantReadNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_target_node(&mut self, node: &ruby_prism::ConstantTargetNode) {
        run_rules!(node, self, ConstantTargetNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_constant_write_node(&mut self, node: &ruby_prism::ConstantWriteNode) {
        run_rules!(node, self, ConstantWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_constant_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_def_node(&mut self, node: &ruby_prism::DefNode) {
        run_rules!(node, self, DefNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_def_node(self, node);
        self.pop_ancestor();
    }
    fn visit_defined_node(&mut self, node: &ruby_prism::DefinedNode) {
        run_rules!(node, self, DefinedNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_defined_node(self, node);
        self.pop_ancestor();
    }
    fn visit_else_node(&mut self, node: &ruby_prism::ElseNode) {
        run_rules!(node, self, ElseNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_else_node(self, node);
        self.pop_ancestor();
    }
    fn visit_embedded_statements_node(&mut self, node: &ruby_prism::EmbeddedStatementsNode) {
        run_rules!(node, self, EmbeddedStatementsNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_embedded_statements_node(self, node);
        self.pop_ancestor();
    }
    fn visit_embedded_variable_node(&mut self, node: &ruby_prism::EmbeddedVariableNode) {
        run_rules!(node, self, EmbeddedVariableNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_embedded_variable_node(self, node);
        self.pop_ancestor();
    }
    fn visit_ensure_node(&mut self, node: &ruby_prism::EnsureNode) {
        run_rules!(node, self, EnsureNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_ensure_node(self, node);
        self.pop_ancestor();
    }
    fn visit_false_node(&mut self, node: &ruby_prism::FalseNode) {
        run_rules!(node, self, FalseNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_false_node(self, node);
        self.pop_ancestor();
    }
    fn visit_find_pattern_node(&mut self, node: &ruby_prism::FindPatternNode) {
        run_rules!(node, self, FindPatternNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_find_pattern_node(self, node);
        self.pop_ancestor();
    }
    fn visit_flip_flop_node(&mut self, node: &ruby_prism::FlipFlopNode) {
        run_rules!(node, self, FlipFlopNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_flip_flop_node(self, node);
        self.pop_ancestor();
    }
    fn visit_float_node(&mut self, node: &ruby_prism::FloatNode) {
        run_rules!(node, self, FloatNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_float_node(self, node);
        self.pop_ancestor();
    }
    fn visit_for_node(&mut self, node: &ruby_prism::ForNode) {
        run_rules!(node, self, ForNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_for_node(self, node);
        self.pop_ancestor();
    }
    fn visit_forwarding_arguments_node(&mut self, node: &ruby_prism::ForwardingArgumentsNode) {
        run_rules!(node, self, ForwardingArgumentsNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_forwarding_arguments_node(self, node);
        self.pop_ancestor();
    }
    fn visit_forwarding_parameter_node(&mut self, node: &ruby_prism::ForwardingParameterNode) {
        run_rules!(node, self, ForwardingParameterNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_forwarding_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_forwarding_super_node(&mut self, node: &ruby_prism::ForwardingSuperNode) {
        run_rules!(node, self, ForwardingSuperNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_forwarding_super_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_and_write_node(&mut self, node: &ruby_prism::GlobalVariableAndWriteNode) {
        run_rules!(node, self, GlobalVariableAndWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_operator_write_node(&mut self, node: &ruby_prism::GlobalVariableOperatorWriteNode) {
        run_rules!(node, self, GlobalVariableOperatorWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_or_write_node(&mut self, node: &ruby_prism::GlobalVariableOrWriteNode) {
        run_rules!(node, self, GlobalVariableOrWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_read_node(&mut self, node: &ruby_prism::GlobalVariableReadNode) {
        run_rules!(node, self, GlobalVariableReadNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_target_node(&mut self, node: &ruby_prism::GlobalVariableTargetNode) {
        run_rules!(node, self, GlobalVariableTargetNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_global_variable_write_node(&mut self, node: &ruby_prism::GlobalVariableWriteNode) {
        run_rules!(node, self, GlobalVariableWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_global_variable_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_hash_node(&mut self, node: &ruby_prism::HashNode) {
        run_rules!(node, self, HashNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_hash_node(self, node);
        self.pop_ancestor();
    }
    fn visit_hash_pattern_node(&mut self, node: &ruby_prism::HashPatternNode) {
        run_rules!(node, self, HashPatternNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_hash_pattern_node(self, node);
        self.pop_ancestor();
    }
    fn visit_if_node(&mut self, node: &ruby_prism::IfNode) {
        run_rules!(node, self, IfNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_if_node(self, node);
        self.pop_ancestor();
    }
    fn visit_imaginary_node(&mut self, node: &ruby_prism::ImaginaryNode) {
        run_rules!(node, self, ImaginaryNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_imaginary_node(self, node);
        self.pop_ancestor();
    }
    fn visit_implicit_node(&mut self, node: &ruby_prism::ImplicitNode) {
        run_rules!(node, self, ImplicitNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_implicit_node(self, node);
        self.pop_ancestor();
    }
    fn visit_implicit_rest_node(&mut self, node: &ruby_prism::ImplicitRestNode) {
        run_rules!(node, self, ImplicitRestNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_implicit_rest_node(self, node);
        self.pop_ancestor();
    }
    fn visit_in_node(&mut self, node: &ruby_prism::InNode) {
        run_rules!(node, self, InNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_in_node(self, node);
        self.pop_ancestor();
    }
    fn visit_index_and_write_node(&mut self, node: &ruby_prism::IndexAndWriteNode) {
        run_rules!(node, self, IndexAndWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_index_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_index_operator_write_node(&mut self, node: &ruby_prism::IndexOperatorWriteNode) {
        run_rules!(node, self, IndexOperatorWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_index_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_index_or_write_node(&mut self, node: &ruby_prism::IndexOrWriteNode) {
        run_rules!(node, self, IndexOrWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_index_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_index_target_node(&mut self, node: &ruby_prism::IndexTargetNode) {
        run_rules!(node, self, IndexTargetNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_index_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_and_write_node(&mut self, node: &ruby_prism::InstanceVariableAndWriteNode) {
        run_rules!(node, self, InstanceVariableAndWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_operator_write_node(&mut self, node: &ruby_prism::InstanceVariableOperatorWriteNode) {
        run_rules!(node, self, InstanceVariableOperatorWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_or_write_node(&mut self, node: &ruby_prism::InstanceVariableOrWriteNode) {
        run_rules!(node, self, InstanceVariableOrWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_read_node(&mut self, node: &ruby_prism::InstanceVariableReadNode) {
        run_rules!(node, self, InstanceVariableReadNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_target_node(&mut self, node: &ruby_prism::InstanceVariableTargetNode) {
        run_rules!(node, self, InstanceVariableTargetNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_instance_variable_write_node(&mut self, node: &ruby_prism::InstanceVariableWriteNode) {
        run_rules!(node, self, InstanceVariableWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_instance_variable_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_integer_node(&mut self, node: &ruby_prism::IntegerNode) {
        run_rules!(node, self, IntegerNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_integer_node(self, node);
        self.pop_ancestor();
    }
    fn visit_interpolated_match_last_line_node(&mut self, node: &ruby_prism::InterpolatedMatchLastLineNode) {
        run_rules!(node, self, InterpolatedMatchLastLineNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_interpolated_match_last_line_node(self, node);
        self.pop_ancestor();
    }
    fn visit_interpolated_regular_expression_node(&mut self, node: &ruby_prism::InterpolatedRegularExpressionNode) {
        run_rules!(node, self, InterpolatedRegularExpressionNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_interpolated_regular_expression_node(self, node);
        self.pop_ancestor();
    }
    fn visit_interpolated_string_node(&mut self, node: &ruby_prism::InterpolatedStringNode) {
        run_rules!(node, self, InterpolatedStringNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_interpolated_string_node(self, node);
        self.pop_ancestor();
    }
    fn visit_interpolated_symbol_node(&mut self, node: &ruby_prism::InterpolatedSymbolNode) {
        run_rules!(node, self, InterpolatedSymbolNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_interpolated_symbol_node(self, node);
        self.pop_ancestor();
    }
    fn visit_interpolated_x_string_node(&mut self, node: &ruby_prism::InterpolatedXStringNode) {
        run_rules!(node, self, InterpolatedXStringNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_interpolated_x_string_node(self, node);
        self.pop_ancestor();
    }
    fn visit_it_local_variable_read_node(&mut self, node: &ruby_prism::ItLocalVariableReadNode) {
        run_rules!(node, self, ItLocalVariableReadNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_it_local_variable_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_it_parameters_node(&mut self, node: &ruby_prism::ItParametersNode) {
        run_rules!(node, self, ItParametersNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_it_parameters_node(self, node);
        self.pop_ancestor();
    }
    fn visit_keyword_hash_node(&mut self, node: &ruby_prism::KeywordHashNode) {
        run_rules!(node, self, KeywordHashNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_keyword_hash_node(self, node);
        self.pop_ancestor();
    }
    fn visit_keyword_rest_parameter_node(&mut self, node: &ruby_prism::KeywordRestParameterNode) {
        run_rules!(node, self, KeywordRestParameterNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_keyword_rest_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode) {
        run_rules!(node, self, LambdaNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_lambda_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_and_write_node(&mut self, node: &ruby_prism::LocalVariableAndWriteNode) {
        run_rules!(node, self, LocalVariableAndWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_and_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_operator_write_node(&mut self, node: &ruby_prism::LocalVariableOperatorWriteNode) {
        run_rules!(node, self, LocalVariableOperatorWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_operator_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_or_write_node(&mut self, node: &ruby_prism::LocalVariableOrWriteNode) {
        run_rules!(node, self, LocalVariableOrWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_or_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode) {
        run_rules!(node, self, LocalVariableReadNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_target_node(&mut self, node: &ruby_prism::LocalVariableTargetNode) {
        run_rules!(node, self, LocalVariableTargetNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode) {
        run_rules!(node, self, LocalVariableWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_local_variable_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_match_last_line_node(&mut self, node: &ruby_prism::MatchLastLineNode) {
        run_rules!(node, self, MatchLastLineNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_match_last_line_node(self, node);
        self.pop_ancestor();
    }
    fn visit_match_predicate_node(&mut self, node: &ruby_prism::MatchPredicateNode) {
        run_rules!(node, self, MatchPredicateNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_match_predicate_node(self, node);
        self.pop_ancestor();
    }
    fn visit_match_required_node(&mut self, node: &ruby_prism::MatchRequiredNode) {
        run_rules!(node, self, MatchRequiredNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_match_required_node(self, node);
        self.pop_ancestor();
    }
    fn visit_match_write_node(&mut self, node: &ruby_prism::MatchWriteNode) {
        run_rules!(node, self, MatchWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_match_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_missing_node(&mut self, node: &ruby_prism::MissingNode) {
        run_rules!(node, self, MissingNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_missing_node(self, node);
        self.pop_ancestor();
    }
    fn visit_module_node(&mut self, node: &ruby_prism::ModuleNode) {
        run_rules!(node, self, ModuleNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_module_node(self, node);
        self.pop_ancestor();
    }
    fn visit_multi_target_node(&mut self, node: &ruby_prism::MultiTargetNode) {
        run_rules!(node, self, MultiTargetNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_multi_target_node(self, node);
        self.pop_ancestor();
    }
    fn visit_multi_write_node(&mut self, node: &ruby_prism::MultiWriteNode) {
        run_rules!(node, self, MultiWriteNode<'_>, []);
        run_rules!(AssignmentNode::from(node), self, AssignmentNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_multi_write_node(self, node);
        self.pop_ancestor();
    }
    fn visit_next_node(&mut self, node: &ruby_prism::NextNode) {
        run_rules!(node, self, NextNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_next_node(self, node);
        self.pop_ancestor();
    }
    fn visit_nil_node(&mut self, node: &ruby_prism::NilNode) {
        run_rules!(node, self, NilNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_nil_node(self, node);
        self.pop_ancestor();
    }
    fn visit_no_keywords_parameter_node(&mut self, node: &ruby_prism::NoKeywordsParameterNode) {
        run_rules!(node, self, NoKeywordsParameterNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_no_keywords_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_numbered_parameters_node(&mut self, node: &ruby_prism::NumberedParametersNode) {
        run_rules!(node, self, NumberedParametersNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_numbered_parameters_node(self, node);
        self.pop_ancestor();
    }
    fn visit_numbered_reference_read_node(&mut self, node: &ruby_prism::NumberedReferenceReadNode) {
        run_rules!(node, self, NumberedReferenceReadNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_numbered_reference_read_node(self, node);
        self.pop_ancestor();
    }
    fn visit_optional_keyword_parameter_node(&mut self, node: &ruby_prism::OptionalKeywordParameterNode) {
        run_rules!(node, self, OptionalKeywordParameterNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_optional_keyword_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_optional_parameter_node(&mut self, node: &ruby_prism::OptionalParameterNode) {
        run_rules!(node, self, OptionalParameterNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_optional_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_or_node(&mut self, node: &ruby_prism::OrNode) {
        run_rules!(node, self, OrNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_or_node(self, node);
        self.pop_ancestor();
    }
    fn visit_parameters_node(&mut self, node: &ruby_prism::ParametersNode) {
        run_rules!(node, self, ParametersNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_parameters_node(self, node);
        self.pop_ancestor();
    }
    fn visit_parentheses_node(&mut self, node: &ruby_prism::ParenthesesNode) {
        run_rules!(node, self, ParenthesesNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_parentheses_node(self, node);
        self.pop_ancestor();
    }
    fn visit_pinned_expression_node(&mut self, node: &ruby_prism::PinnedExpressionNode) {
        run_rules!(node, self, PinnedExpressionNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_pinned_expression_node(self, node);
        self.pop_ancestor();
    }
    fn visit_pinned_variable_node(&mut self, node: &ruby_prism::PinnedVariableNode) {
        run_rules!(node, self, PinnedVariableNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_pinned_variable_node(self, node);
        self.pop_ancestor();
    }
    fn visit_post_execution_node(&mut self, node: &ruby_prism::PostExecutionNode) {
        run_rules!(node, self, PostExecutionNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_post_execution_node(self, node);
        self.pop_ancestor();
    }
    fn visit_pre_execution_node(&mut self, node: &ruby_prism::PreExecutionNode) {
        run_rules!(node, self, PreExecutionNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_pre_execution_node(self, node);
        self.pop_ancestor();
    }
    fn visit_program_node(&mut self, node: &ruby_prism::ProgramNode) {
        run_rules!(node, self, ProgramNode<'_>, []);
        // self.push_ancestor(node.as_node());
        ruby_prism::visit_program_node(self, node);
        // self.pop_ancestor();
    }
    fn visit_range_node(&mut self, node: &ruby_prism::RangeNode) {
        run_rules!(node, self, RangeNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_range_node(self, node);
        self.pop_ancestor();
    }
    fn visit_rational_node(&mut self, node: &ruby_prism::RationalNode) {
        run_rules!(node, self, RationalNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_rational_node(self, node);
        self.pop_ancestor();
    }
    fn visit_redo_node(&mut self, node: &ruby_prism::RedoNode) {
        run_rules!(node, self, RedoNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_redo_node(self, node);
        self.pop_ancestor();
    }
    fn visit_regular_expression_node(&mut self, node: &ruby_prism::RegularExpressionNode) {
        run_rules!(node, self, RegularExpressionNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_regular_expression_node(self, node);
        self.pop_ancestor();
    }
    fn visit_required_keyword_parameter_node(&mut self, node: &ruby_prism::RequiredKeywordParameterNode) {
        run_rules!(node, self, RequiredKeywordParameterNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_required_keyword_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_required_parameter_node(&mut self, node: &ruby_prism::RequiredParameterNode) {
        run_rules!(node, self, RequiredParameterNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_required_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_rescue_modifier_node(&mut self, node: &ruby_prism::RescueModifierNode) {
        run_rules!(node, self, RescueModifierNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_rescue_modifier_node(self, node);
        self.pop_ancestor();
    }
    fn visit_rescue_node(&mut self, node: &ruby_prism::RescueNode) {
        run_rules!(node, self, RescueNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_rescue_node(self, node);
        self.pop_ancestor();
    }
    fn visit_rest_parameter_node(&mut self, node: &ruby_prism::RestParameterNode) {
        run_rules!(node, self, RestParameterNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_rest_parameter_node(self, node);
        self.pop_ancestor();
    }
    fn visit_retry_node(&mut self, node: &ruby_prism::RetryNode) {
        run_rules!(node, self, RetryNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_retry_node(self, node);
        self.pop_ancestor();
    }
    fn visit_return_node(&mut self, node: &ruby_prism::ReturnNode) {
        run_rules!(node, self, ReturnNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_return_node(self, node);
        self.pop_ancestor();
    }
    fn visit_self_node(&mut self, node: &ruby_prism::SelfNode) {
        run_rules!(node, self, SelfNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_self_node(self, node);
        self.pop_ancestor();
    }
    fn visit_shareable_constant_node(&mut self, node: &ruby_prism::ShareableConstantNode) {
        run_rules!(node, self, ShareableConstantNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_shareable_constant_node(self, node);
        self.pop_ancestor();
    }
    fn visit_singleton_class_node(&mut self, node: &ruby_prism::SingletonClassNode) {
        run_rules!(node, self, SingletonClassNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_singleton_class_node(self, node);
        self.pop_ancestor();
    }
    fn visit_source_encoding_node(&mut self, node: &ruby_prism::SourceEncodingNode) {
        run_rules!(node, self, SourceEncodingNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_source_encoding_node(self, node);
        self.pop_ancestor();
    }
    fn visit_source_file_node(&mut self, node: &ruby_prism::SourceFileNode) {
        run_rules!(node, self, SourceFileNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_source_file_node(self, node);
        self.pop_ancestor();
    }
    fn visit_source_line_node(&mut self, node: &ruby_prism::SourceLineNode) {
        run_rules!(node, self, SourceLineNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_source_line_node(self, node);
        self.pop_ancestor();
    }
    fn visit_splat_node(&mut self, node: &ruby_prism::SplatNode) {
        run_rules!(node, self, SplatNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_splat_node(self, node);
        self.pop_ancestor();
    }
    fn visit_statements_node(&mut self, node: &ruby_prism::StatementsNode) {
        run_rules!(node, self, StatementsNode<'_>, []);
        // Skip adding ProgramNode's StatementsNode to ancestors
        // (ProgramNode is the root and not pushed to ancestors)
        let should_track = self.parent().is_some();
        should_track.then(|| self.push_ancestor(node.as_node()));
        ruby_prism::visit_statements_node(self, node);
        should_track.then(|| self.pop_ancestor());
    }
    fn visit_string_node(&mut self, node: &ruby_prism::StringNode) {
        run_rules!(node, self, StringNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_string_node(self, node);
        self.pop_ancestor();
    }
    fn visit_super_node(&mut self, node: &ruby_prism::SuperNode) {
        run_rules!(node, self, SuperNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_super_node(self, node);
        self.pop_ancestor();
    }
    fn visit_symbol_node(&mut self, node: &ruby_prism::SymbolNode) {
        run_rules!(node, self, SymbolNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_symbol_node(self, node);
        self.pop_ancestor();
    }
    fn visit_true_node(&mut self, node: &ruby_prism::TrueNode) {
        run_rules!(node, self, TrueNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_true_node(self, node);
        self.pop_ancestor();
    }
    fn visit_undef_node(&mut self, node: &ruby_prism::UndefNode) {
        run_rules!(node, self, UndefNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_undef_node(self, node);
        self.pop_ancestor();
    }
    fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode) {
        run_rules!(node, self, UnlessNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_unless_node(self, node);
        self.pop_ancestor();
    }
    fn visit_until_node(&mut self, node: &ruby_prism::UntilNode) {
        run_rules!(node, self, UntilNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_until_node(self, node);
        self.pop_ancestor();
    }
    fn visit_when_node(&mut self, node: &ruby_prism::WhenNode) {
        run_rules!(node, self, WhenNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_when_node(self, node);
        self.pop_ancestor();
    }
    fn visit_while_node(&mut self, node: &ruby_prism::WhileNode) {
        run_rules!(node, self, WhileNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_while_node(self, node);
        self.pop_ancestor();
    }
    fn visit_x_string_node(&mut self, node: &ruby_prism::XStringNode) {
        run_rules!(node, self, XStringNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_x_string_node(self, node);
        self.pop_ancestor();
    }
    fn visit_yield_node(&mut self, node: &ruby_prism::YieldNode) {
        run_rules!(node, self, YieldNode<'_>, []);
        self.push_ancestor(node.as_node());
        ruby_prism::visit_yield_node(self, node);
        self.pop_ancestor();
    }
}
