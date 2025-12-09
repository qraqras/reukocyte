use crate::config::Config;
use crate::custom_nodes::AssignmentNode;
use crate::diagnostic::Diagnostic;
use crate::diagnostic::Fix;
use crate::diagnostic::RawDiagnostic;
use crate::diagnostic::Severity;
use crate::locator::LineIndex;
use crate::rule::RuleId;
use crate::semantic::SemanticModel;
use ruby_prism::*;
use rustc_hash::FxHashSet;

// Include the auto-generated rule registry macros
include!(concat!(env!("OUT_DIR"), "/rule_registry.rs"));

/// A visitor that builds the node index before rules run.
/// This ensures all nodes have assigned IDs for rules to reference.
struct IndexingVisitor<'rk, 'checker> {
    semantic: &'checker mut SemanticModel<'rk>,
}
impl<'rk> Visit<'rk> for IndexingVisitor<'rk, '_> {
    fn visit_branch_node_enter(&mut self, node: Node<'rk>) {
        self.semantic.push_node(node);
    }
    fn visit_branch_node_leave(&mut self) {
        self.semantic.pop_node();
    }
    fn visit_leaf_node_enter(&mut self, node: Node<'rk>) {
        self.semantic.push_node(node);
    }
    fn visit_leaf_node_leave(&mut self) {
        self.semantic.pop_node();
    }
}

/// The main checker that traverses the AST and runs rules.
pub struct Checker<'rk> {
    source: &'rk [u8],
    config: &'rk Config,
    ignored_nodes: FxHashSet<(usize, usize)>,
    line_index: LineIndex<'rk>,
    raw_diagnostics: Vec<RawDiagnostic>,
    semantic: SemanticModel<'rk>,
}
impl<'rk> Checker<'rk> {
    /// Create a new Checker instance.
    pub fn new(source: &'rk [u8], config: &'rk Config) -> Self {
        Self {
            source,
            config,
            ignored_nodes: FxHashSet::default(),
            line_index: LineIndex::from_source(source),
            raw_diagnostics: Vec::new(),
            semantic: SemanticModel::new(),
        }
    }
    /// Build the node index by traversing the AST before running rules.
    pub fn build_index(&mut self, root: &Node<'rk>) {
        let mut visitor = IndexingVisitor { semantic: &mut self.semantic };
        visitor.visit(root);
    }
    /// Get the source code being checked.
    #[inline]
    pub fn source(&self) -> &[u8] {
        self.source
    }
    /// Get the configuration used by the checker.
    #[inline]
    pub fn config(&self) -> &Config {
        self.config
    }
    /// Get the line index for offset-to-line/column mapping.
    #[inline]
    pub fn line_index(&self) -> &LineIndex<'rk> {
        &self.line_index
    }
    /// Get access to the semantic model.
    #[inline]
    pub fn semantic(&self) -> &SemanticModel<'rk> {
        &self.semantic
    }

    // ======== Rule management ==========

    /// Check if a rule is enabled.
    ///
    /// TODO: Implement rule enable/disable logic based on config.
    /// For now, all rules are enabled.
    #[inline]
    pub fn is_enabled(&self, _rule_id: RuleId) -> bool {
        true
    }

    // ========= Node stack management ==========

    /// Push a node onto the semantic model (called before visiting children).
    #[inline]
    fn push_node(&mut self, node: Node<'_>) {
        // SAFETY: We know the node lifetime is valid during the visit traversal.
        // The semantic model will be dropped before the source is invalidated.
        let node: Node<'rk> = unsafe { std::mem::transmute(node) };
        self.semantic.push_node(node);
    }
    /// Pop the current node from the semantic model (called after visiting children).
    #[inline]
    fn pop_node(&mut self) {
        self.semantic.pop_node();
    }

    // ======== Ignored nodes management ==========

    /// Mark a node as ignored (will not be processed by rules).
    #[inline]
    pub fn ignore_node(&mut self, location: &Location) {
        self.ignored_nodes.insert((location.start_offset(), location.end_offset()));
    }
    /// Check if a node is exactly one of the ignored nodes.
    #[inline]
    pub fn is_ignored_node(&self, start_offset: usize, end_offset: usize) -> bool {
        self.ignored_nodes.contains(&(start_offset, end_offset))
    }

    // ========= Diagnostic reporting ==========

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
        self.raw_diagnostics.sort_by_key(|d| (d.start, d.end));
        let offsets: Vec<(usize, usize)> = self.raw_diagnostics.iter().map(|d| (d.start, d.end)).collect();
        let resolved = self.line_index.batch_line_column(&offsets);
        self.raw_diagnostics
            .into_iter()
            .zip(resolved)
            .map(|(raw, (line_start, line_end, column_start, column_end))| raw.resolve(line_start, line_end, column_start, column_end))
            .collect()
    }
}

impl Visit<'_> for Checker<'_> {
    fn visit_alias_global_variable_node(&mut self, node: &ruby_prism::AliasGlobalVariableNode) {
        self.push_node(node.as_node());
        run_alias_global_variable_node_rules!(node, self);
        ruby_prism::visit_alias_global_variable_node(self, node);
        self.pop_node();
    }
    fn visit_alias_method_node(&mut self, node: &ruby_prism::AliasMethodNode) {
        self.push_node(node.as_node());
        run_alias_method_node_rules!(node, self);
        ruby_prism::visit_alias_method_node(self, node);
        self.pop_node();
    }
    fn visit_alternation_pattern_node(&mut self, node: &ruby_prism::AlternationPatternNode) {
        self.push_node(node.as_node());
        run_alternation_pattern_node_rules!(node, self);
        ruby_prism::visit_alternation_pattern_node(self, node);
        self.pop_node();
    }
    fn visit_and_node(&mut self, node: &ruby_prism::AndNode) {
        self.push_node(node.as_node());
        run_and_node_rules!(node, self);
        ruby_prism::visit_and_node(self, node);
        self.pop_node();
    }
    fn visit_arguments_node(&mut self, node: &ruby_prism::ArgumentsNode) {
        self.push_node(node.as_node());
        run_arguments_node_rules!(node, self);
        ruby_prism::visit_arguments_node(self, node);
        self.pop_node();
    }
    fn visit_array_node(&mut self, node: &ruby_prism::ArrayNode) {
        self.push_node(node.as_node());
        run_array_node_rules!(node, self);
        ruby_prism::visit_array_node(self, node);
        self.pop_node();
    }
    fn visit_array_pattern_node(&mut self, node: &ruby_prism::ArrayPatternNode) {
        self.push_node(node.as_node());
        run_array_pattern_node_rules!(node, self);
        ruby_prism::visit_array_pattern_node(self, node);
        self.pop_node();
    }
    fn visit_assoc_node(&mut self, node: &ruby_prism::AssocNode) {
        self.push_node(node.as_node());
        run_assoc_node_rules!(node, self);
        ruby_prism::visit_assoc_node(self, node);
        self.pop_node();
    }
    fn visit_assoc_splat_node(&mut self, node: &ruby_prism::AssocSplatNode) {
        self.push_node(node.as_node());
        run_assoc_splat_node_rules!(node, self);
        ruby_prism::visit_assoc_splat_node(self, node);
        self.pop_node();
    }
    fn visit_back_reference_read_node(&mut self, node: &ruby_prism::BackReferenceReadNode) {
        self.push_node(node.as_node());
        run_back_reference_read_node_rules!(node, self);
        ruby_prism::visit_back_reference_read_node(self, node);
        self.pop_node();
    }
    fn visit_begin_node(&mut self, node: &ruby_prism::BeginNode) {
        self.push_node(node.as_node());
        run_begin_node_rules!(node, self);
        ruby_prism::visit_begin_node(self, node);
        self.pop_node();
    }
    fn visit_block_argument_node(&mut self, node: &ruby_prism::BlockArgumentNode) {
        self.push_node(node.as_node());
        run_block_argument_node_rules!(node, self);
        ruby_prism::visit_block_argument_node(self, node);
        self.pop_node();
    }
    fn visit_block_local_variable_node(&mut self, node: &ruby_prism::BlockLocalVariableNode) {
        self.push_node(node.as_node());
        run_block_local_variable_node_rules!(node, self);
        ruby_prism::visit_block_local_variable_node(self, node);
        self.pop_node();
    }
    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode) {
        self.push_node(node.as_node());
        run_block_node_rules!(node, self);
        ruby_prism::visit_block_node(self, node);
        self.pop_node();
    }
    fn visit_block_parameter_node(&mut self, node: &ruby_prism::BlockParameterNode) {
        self.push_node(node.as_node());
        run_block_parameter_node_rules!(node, self);
        ruby_prism::visit_block_parameter_node(self, node);
        self.pop_node();
    }
    fn visit_block_parameters_node(&mut self, node: &ruby_prism::BlockParametersNode) {
        self.push_node(node.as_node());
        run_block_parameters_node_rules!(node, self);
        ruby_prism::visit_block_parameters_node(self, node);
        self.pop_node();
    }
    fn visit_break_node(&mut self, node: &ruby_prism::BreakNode) {
        self.push_node(node.as_node());
        run_break_node_rules!(node, self);
        ruby_prism::visit_break_node(self, node);
        self.pop_node();
    }
    fn visit_call_and_write_node(&mut self, node: &ruby_prism::CallAndWriteNode) {
        self.push_node(node.as_node());
        run_call_and_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_call_and_write_node(self, node);
        self.pop_node();
    }
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode) {
        self.push_node(node.as_node());
        run_call_node_rules!(node, self);
        ruby_prism::visit_call_node(self, node);
        self.pop_node();
    }
    fn visit_call_operator_write_node(&mut self, node: &ruby_prism::CallOperatorWriteNode) {
        self.push_node(node.as_node());
        run_call_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_call_operator_write_node(self, node);
        self.pop_node();
    }
    fn visit_call_or_write_node(&mut self, node: &ruby_prism::CallOrWriteNode) {
        self.push_node(node.as_node());
        run_call_or_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_call_or_write_node(self, node);
        self.pop_node();
    }
    fn visit_call_target_node(&mut self, node: &ruby_prism::CallTargetNode) {
        self.push_node(node.as_node());
        run_call_target_node_rules!(node, self);
        ruby_prism::visit_call_target_node(self, node);
        self.pop_node();
    }
    fn visit_capture_pattern_node(&mut self, node: &ruby_prism::CapturePatternNode) {
        self.push_node(node.as_node());
        run_capture_pattern_node_rules!(node, self);
        ruby_prism::visit_capture_pattern_node(self, node);
        self.pop_node();
    }
    fn visit_case_match_node(&mut self, node: &ruby_prism::CaseMatchNode) {
        self.push_node(node.as_node());
        run_case_match_node_rules!(node, self);
        ruby_prism::visit_case_match_node(self, node);
        self.pop_node();
    }
    fn visit_case_node(&mut self, node: &ruby_prism::CaseNode) {
        self.push_node(node.as_node());
        run_case_node_rules!(node, self);
        ruby_prism::visit_case_node(self, node);
        self.pop_node();
    }
    fn visit_class_node(&mut self, node: &ruby_prism::ClassNode) {
        self.push_node(node.as_node());
        run_class_node_rules!(node, self);
        ruby_prism::visit_class_node(self, node);
        self.pop_node();
    }
    fn visit_class_variable_and_write_node(&mut self, node: &ruby_prism::ClassVariableAndWriteNode) {
        self.push_node(node.as_node());
        run_class_variable_and_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_class_variable_and_write_node(self, node);
        self.pop_node();
    }
    fn visit_class_variable_operator_write_node(&mut self, node: &ruby_prism::ClassVariableOperatorWriteNode) {
        self.push_node(node.as_node());
        run_class_variable_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_class_variable_operator_write_node(self, node);
        self.pop_node();
    }
    fn visit_class_variable_or_write_node(&mut self, node: &ruby_prism::ClassVariableOrWriteNode) {
        self.push_node(node.as_node());
        run_class_variable_or_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_class_variable_or_write_node(self, node);
        self.pop_node();
    }
    fn visit_class_variable_read_node(&mut self, node: &ruby_prism::ClassVariableReadNode) {
        self.push_node(node.as_node());
        run_class_variable_read_node_rules!(node, self);
        ruby_prism::visit_class_variable_read_node(self, node);
        self.pop_node();
    }
    fn visit_class_variable_target_node(&mut self, node: &ruby_prism::ClassVariableTargetNode) {
        self.push_node(node.as_node());
        run_class_variable_target_node_rules!(node, self);
        ruby_prism::visit_class_variable_target_node(self, node);
        self.pop_node();
    }
    fn visit_class_variable_write_node(&mut self, node: &ruby_prism::ClassVariableWriteNode) {
        self.push_node(node.as_node());
        run_class_variable_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_class_variable_write_node(self, node);
        self.pop_node();
    }
    fn visit_constant_and_write_node(&mut self, node: &ruby_prism::ConstantAndWriteNode) {
        self.push_node(node.as_node());
        run_constant_and_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_constant_and_write_node(self, node);
        self.pop_node();
    }
    fn visit_constant_operator_write_node(&mut self, node: &ruby_prism::ConstantOperatorWriteNode) {
        self.push_node(node.as_node());
        run_constant_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_constant_operator_write_node(self, node);
        self.pop_node();
    }
    fn visit_constant_or_write_node(&mut self, node: &ruby_prism::ConstantOrWriteNode) {
        self.push_node(node.as_node());
        run_constant_or_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_constant_or_write_node(self, node);
        self.pop_node();
    }
    fn visit_constant_path_and_write_node(&mut self, node: &ruby_prism::ConstantPathAndWriteNode) {
        self.push_node(node.as_node());
        run_constant_path_and_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_constant_path_and_write_node(self, node);
        self.pop_node();
    }
    fn visit_constant_path_node(&mut self, node: &ruby_prism::ConstantPathNode) {
        self.push_node(node.as_node());
        run_constant_path_node_rules!(node, self);
        ruby_prism::visit_constant_path_node(self, node);
        self.pop_node();
    }
    fn visit_constant_path_operator_write_node(&mut self, node: &ruby_prism::ConstantPathOperatorWriteNode) {
        self.push_node(node.as_node());
        run_constant_path_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_constant_path_operator_write_node(self, node);
        self.pop_node();
    }
    fn visit_constant_path_or_write_node(&mut self, node: &ruby_prism::ConstantPathOrWriteNode) {
        self.push_node(node.as_node());
        run_constant_path_or_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_constant_path_or_write_node(self, node);
        self.pop_node();
    }
    fn visit_constant_path_target_node(&mut self, node: &ruby_prism::ConstantPathTargetNode) {
        self.push_node(node.as_node());
        run_constant_path_target_node_rules!(node, self);
        ruby_prism::visit_constant_path_target_node(self, node);
        self.pop_node();
    }
    fn visit_constant_path_write_node(&mut self, node: &ruby_prism::ConstantPathWriteNode) {
        self.push_node(node.as_node());
        run_constant_path_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_constant_path_write_node(self, node);
        self.pop_node();
    }
    fn visit_constant_read_node(&mut self, node: &ruby_prism::ConstantReadNode) {
        self.push_node(node.as_node());
        run_constant_read_node_rules!(node, self);
        ruby_prism::visit_constant_read_node(self, node);
        self.pop_node();
    }
    fn visit_constant_target_node(&mut self, node: &ruby_prism::ConstantTargetNode) {
        self.push_node(node.as_node());
        run_constant_target_node_rules!(node, self);
        ruby_prism::visit_constant_target_node(self, node);
        self.pop_node();
    }
    fn visit_constant_write_node(&mut self, node: &ruby_prism::ConstantWriteNode) {
        self.push_node(node.as_node());
        run_constant_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_constant_write_node(self, node);
        self.pop_node();
    }
    fn visit_def_node(&mut self, node: &ruby_prism::DefNode) {
        self.push_node(node.as_node());
        run_def_node_rules!(node, self);
        ruby_prism::visit_def_node(self, node);
        self.pop_node();
    }
    fn visit_defined_node(&mut self, node: &ruby_prism::DefinedNode) {
        self.push_node(node.as_node());
        run_defined_node_rules!(node, self);
        ruby_prism::visit_defined_node(self, node);
        self.pop_node();
    }
    fn visit_else_node(&mut self, node: &ruby_prism::ElseNode) {
        self.push_node(node.as_node());
        run_else_node_rules!(node, self);
        ruby_prism::visit_else_node(self, node);
        self.pop_node();
    }
    fn visit_embedded_statements_node(&mut self, node: &ruby_prism::EmbeddedStatementsNode) {
        self.push_node(node.as_node());
        run_embedded_statements_node_rules!(node, self);
        ruby_prism::visit_embedded_statements_node(self, node);
        self.pop_node();
    }
    fn visit_embedded_variable_node(&mut self, node: &ruby_prism::EmbeddedVariableNode) {
        self.push_node(node.as_node());
        run_embedded_variable_node_rules!(node, self);
        ruby_prism::visit_embedded_variable_node(self, node);
        self.pop_node();
    }
    fn visit_ensure_node(&mut self, node: &ruby_prism::EnsureNode) {
        self.push_node(node.as_node());
        run_ensure_node_rules!(node, self);
        ruby_prism::visit_ensure_node(self, node);
        self.pop_node();
    }
    fn visit_false_node(&mut self, node: &ruby_prism::FalseNode) {
        self.push_node(node.as_node());
        run_false_node_rules!(node, self);
        ruby_prism::visit_false_node(self, node);
        self.pop_node();
    }
    fn visit_find_pattern_node(&mut self, node: &ruby_prism::FindPatternNode) {
        self.push_node(node.as_node());
        run_find_pattern_node_rules!(node, self);
        ruby_prism::visit_find_pattern_node(self, node);
        self.pop_node();
    }
    fn visit_flip_flop_node(&mut self, node: &ruby_prism::FlipFlopNode) {
        self.push_node(node.as_node());
        run_flip_flop_node_rules!(node, self);
        ruby_prism::visit_flip_flop_node(self, node);
        self.pop_node();
    }
    fn visit_float_node(&mut self, node: &ruby_prism::FloatNode) {
        self.push_node(node.as_node());
        run_float_node_rules!(node, self);
        ruby_prism::visit_float_node(self, node);
        self.pop_node();
    }
    fn visit_for_node(&mut self, node: &ruby_prism::ForNode) {
        self.push_node(node.as_node());
        run_for_node_rules!(node, self);
        ruby_prism::visit_for_node(self, node);
        self.pop_node();
    }
    fn visit_forwarding_arguments_node(&mut self, node: &ruby_prism::ForwardingArgumentsNode) {
        self.push_node(node.as_node());
        run_forwarding_arguments_node_rules!(node, self);
        ruby_prism::visit_forwarding_arguments_node(self, node);
        self.pop_node();
    }
    fn visit_forwarding_parameter_node(&mut self, node: &ruby_prism::ForwardingParameterNode) {
        self.push_node(node.as_node());
        run_forwarding_parameter_node_rules!(node, self);
        ruby_prism::visit_forwarding_parameter_node(self, node);
        self.pop_node();
    }
    fn visit_forwarding_super_node(&mut self, node: &ruby_prism::ForwardingSuperNode) {
        self.push_node(node.as_node());
        run_forwarding_super_node_rules!(node, self);
        ruby_prism::visit_forwarding_super_node(self, node);
        self.pop_node();
    }
    fn visit_global_variable_and_write_node(&mut self, node: &ruby_prism::GlobalVariableAndWriteNode) {
        self.push_node(node.as_node());
        run_global_variable_and_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_global_variable_and_write_node(self, node);
        self.pop_node();
    }
    fn visit_global_variable_operator_write_node(&mut self, node: &ruby_prism::GlobalVariableOperatorWriteNode) {
        self.push_node(node.as_node());
        run_global_variable_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_global_variable_operator_write_node(self, node);
        self.pop_node();
    }
    fn visit_global_variable_or_write_node(&mut self, node: &ruby_prism::GlobalVariableOrWriteNode) {
        self.push_node(node.as_node());
        run_global_variable_or_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_global_variable_or_write_node(self, node);
        self.pop_node();
    }
    fn visit_global_variable_read_node(&mut self, node: &ruby_prism::GlobalVariableReadNode) {
        self.push_node(node.as_node());
        run_global_variable_read_node_rules!(node, self);
        ruby_prism::visit_global_variable_read_node(self, node);
        self.pop_node();
    }
    fn visit_global_variable_target_node(&mut self, node: &ruby_prism::GlobalVariableTargetNode) {
        self.push_node(node.as_node());
        run_global_variable_target_node_rules!(node, self);
        ruby_prism::visit_global_variable_target_node(self, node);
        self.pop_node();
    }
    fn visit_global_variable_write_node(&mut self, node: &ruby_prism::GlobalVariableWriteNode) {
        self.push_node(node.as_node());
        run_global_variable_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_global_variable_write_node(self, node);
        self.pop_node();
    }
    fn visit_hash_node(&mut self, node: &ruby_prism::HashNode) {
        self.push_node(node.as_node());
        run_hash_node_rules!(node, self);
        ruby_prism::visit_hash_node(self, node);
        self.pop_node();
    }
    fn visit_hash_pattern_node(&mut self, node: &ruby_prism::HashPatternNode) {
        self.push_node(node.as_node());
        run_hash_pattern_node_rules!(node, self);
        ruby_prism::visit_hash_pattern_node(self, node);
        self.pop_node();
    }
    fn visit_if_node(&mut self, node: &ruby_prism::IfNode) {
        self.push_node(node.as_node());
        run_if_node_rules!(node, self);
        ruby_prism::visit_if_node(self, node);
        self.pop_node();
    }
    fn visit_imaginary_node(&mut self, node: &ruby_prism::ImaginaryNode) {
        self.push_node(node.as_node());
        run_imaginary_node_rules!(node, self);
        ruby_prism::visit_imaginary_node(self, node);
        self.pop_node();
    }
    fn visit_implicit_node(&mut self, node: &ruby_prism::ImplicitNode) {
        self.push_node(node.as_node());
        run_implicit_node_rules!(node, self);
        ruby_prism::visit_implicit_node(self, node);
        self.pop_node();
    }
    fn visit_implicit_rest_node(&mut self, node: &ruby_prism::ImplicitRestNode) {
        self.push_node(node.as_node());
        run_implicit_rest_node_rules!(node, self);
        ruby_prism::visit_implicit_rest_node(self, node);
        self.pop_node();
    }
    fn visit_in_node(&mut self, node: &ruby_prism::InNode) {
        self.push_node(node.as_node());
        run_in_node_rules!(node, self);
        ruby_prism::visit_in_node(self, node);
        self.pop_node();
    }
    fn visit_index_and_write_node(&mut self, node: &ruby_prism::IndexAndWriteNode) {
        self.push_node(node.as_node());
        run_index_and_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_index_and_write_node(self, node);
        self.pop_node();
    }
    fn visit_index_operator_write_node(&mut self, node: &ruby_prism::IndexOperatorWriteNode) {
        self.push_node(node.as_node());
        run_index_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_index_operator_write_node(self, node);
        self.pop_node();
    }
    fn visit_index_or_write_node(&mut self, node: &ruby_prism::IndexOrWriteNode) {
        self.push_node(node.as_node());
        run_index_or_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_index_or_write_node(self, node);
        self.pop_node();
    }
    fn visit_index_target_node(&mut self, node: &ruby_prism::IndexTargetNode) {
        self.push_node(node.as_node());
        run_index_target_node_rules!(node, self);
        ruby_prism::visit_index_target_node(self, node);
        self.pop_node();
    }
    fn visit_instance_variable_and_write_node(&mut self, node: &ruby_prism::InstanceVariableAndWriteNode) {
        self.push_node(node.as_node());
        run_instance_variable_and_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_instance_variable_and_write_node(self, node);
        self.pop_node();
    }
    fn visit_instance_variable_operator_write_node(&mut self, node: &ruby_prism::InstanceVariableOperatorWriteNode) {
        self.push_node(node.as_node());
        run_instance_variable_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_instance_variable_operator_write_node(self, node);
        self.pop_node();
    }
    fn visit_instance_variable_or_write_node(&mut self, node: &ruby_prism::InstanceVariableOrWriteNode) {
        self.push_node(node.as_node());
        run_instance_variable_or_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_instance_variable_or_write_node(self, node);
        self.pop_node();
    }
    fn visit_instance_variable_read_node(&mut self, node: &ruby_prism::InstanceVariableReadNode) {
        self.push_node(node.as_node());
        run_instance_variable_read_node_rules!(node, self);
        ruby_prism::visit_instance_variable_read_node(self, node);
        self.pop_node();
    }
    fn visit_instance_variable_target_node(&mut self, node: &ruby_prism::InstanceVariableTargetNode) {
        self.push_node(node.as_node());
        run_instance_variable_target_node_rules!(node, self);
        ruby_prism::visit_instance_variable_target_node(self, node);
        self.pop_node();
    }
    fn visit_instance_variable_write_node(&mut self, node: &ruby_prism::InstanceVariableWriteNode) {
        self.push_node(node.as_node());
        run_instance_variable_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_instance_variable_write_node(self, node);
        self.pop_node();
    }
    fn visit_integer_node(&mut self, node: &ruby_prism::IntegerNode) {
        self.push_node(node.as_node());
        run_integer_node_rules!(node, self);
        ruby_prism::visit_integer_node(self, node);
        self.pop_node();
    }
    fn visit_interpolated_match_last_line_node(&mut self, node: &ruby_prism::InterpolatedMatchLastLineNode) {
        self.push_node(node.as_node());
        run_interpolated_match_last_line_node_rules!(node, self);
        ruby_prism::visit_interpolated_match_last_line_node(self, node);
        self.pop_node();
    }
    fn visit_interpolated_regular_expression_node(&mut self, node: &ruby_prism::InterpolatedRegularExpressionNode) {
        self.push_node(node.as_node());
        run_interpolated_regular_expression_node_rules!(node, self);
        ruby_prism::visit_interpolated_regular_expression_node(self, node);
        self.pop_node();
    }
    fn visit_interpolated_string_node(&mut self, node: &ruby_prism::InterpolatedStringNode) {
        self.push_node(node.as_node());
        run_interpolated_string_node_rules!(node, self);
        ruby_prism::visit_interpolated_string_node(self, node);
        self.pop_node();
    }
    fn visit_interpolated_symbol_node(&mut self, node: &ruby_prism::InterpolatedSymbolNode) {
        self.push_node(node.as_node());
        run_interpolated_symbol_node_rules!(node, self);
        ruby_prism::visit_interpolated_symbol_node(self, node);
        self.pop_node();
    }
    fn visit_interpolated_x_string_node(&mut self, node: &ruby_prism::InterpolatedXStringNode) {
        self.push_node(node.as_node());
        run_interpolated_x_string_node_rules!(node, self);
        ruby_prism::visit_interpolated_x_string_node(self, node);
        self.pop_node();
    }
    fn visit_it_local_variable_read_node(&mut self, node: &ruby_prism::ItLocalVariableReadNode) {
        self.push_node(node.as_node());
        run_it_local_variable_read_node_rules!(node, self);
        ruby_prism::visit_it_local_variable_read_node(self, node);
        self.pop_node();
    }
    fn visit_it_parameters_node(&mut self, node: &ruby_prism::ItParametersNode) {
        self.push_node(node.as_node());
        run_it_parameters_node_rules!(node, self);
        ruby_prism::visit_it_parameters_node(self, node);
        self.pop_node();
    }
    fn visit_keyword_hash_node(&mut self, node: &ruby_prism::KeywordHashNode) {
        self.push_node(node.as_node());
        run_keyword_hash_node_rules!(node, self);
        ruby_prism::visit_keyword_hash_node(self, node);
        self.pop_node();
    }
    fn visit_keyword_rest_parameter_node(&mut self, node: &ruby_prism::KeywordRestParameterNode) {
        self.push_node(node.as_node());
        run_keyword_rest_parameter_node_rules!(node, self);
        ruby_prism::visit_keyword_rest_parameter_node(self, node);
        self.pop_node();
    }
    fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode) {
        self.push_node(node.as_node());
        run_lambda_node_rules!(node, self);
        ruby_prism::visit_lambda_node(self, node);
        self.pop_node();
    }
    fn visit_local_variable_and_write_node(&mut self, node: &ruby_prism::LocalVariableAndWriteNode) {
        self.push_node(node.as_node());
        run_local_variable_and_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_local_variable_and_write_node(self, node);
        self.pop_node();
    }
    fn visit_local_variable_operator_write_node(&mut self, node: &ruby_prism::LocalVariableOperatorWriteNode) {
        self.push_node(node.as_node());
        run_local_variable_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_local_variable_operator_write_node(self, node);
        self.pop_node();
    }
    fn visit_local_variable_or_write_node(&mut self, node: &ruby_prism::LocalVariableOrWriteNode) {
        self.push_node(node.as_node());
        run_local_variable_or_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_local_variable_or_write_node(self, node);
        self.pop_node();
    }
    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode) {
        self.push_node(node.as_node());
        run_local_variable_read_node_rules!(node, self);
        ruby_prism::visit_local_variable_read_node(self, node);
        self.pop_node();
    }
    fn visit_local_variable_target_node(&mut self, node: &ruby_prism::LocalVariableTargetNode) {
        self.push_node(node.as_node());
        run_local_variable_target_node_rules!(node, self);
        ruby_prism::visit_local_variable_target_node(self, node);
        self.pop_node();
    }
    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode) {
        self.push_node(node.as_node());
        run_local_variable_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_local_variable_write_node(self, node);
        self.pop_node();
    }
    fn visit_match_last_line_node(&mut self, node: &ruby_prism::MatchLastLineNode) {
        self.push_node(node.as_node());
        run_match_last_line_node_rules!(node, self);
        ruby_prism::visit_match_last_line_node(self, node);
        self.pop_node();
    }
    fn visit_match_predicate_node(&mut self, node: &ruby_prism::MatchPredicateNode) {
        self.push_node(node.as_node());
        run_match_predicate_node_rules!(node, self);
        ruby_prism::visit_match_predicate_node(self, node);
        self.pop_node();
    }
    fn visit_match_required_node(&mut self, node: &ruby_prism::MatchRequiredNode) {
        self.push_node(node.as_node());
        run_match_required_node_rules!(node, self);
        ruby_prism::visit_match_required_node(self, node);
        self.pop_node();
    }
    fn visit_match_write_node(&mut self, node: &ruby_prism::MatchWriteNode) {
        run_match_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.push_node(node.as_node());
        ruby_prism::visit_match_write_node(self, node);
        self.pop_node();
    }
    fn visit_missing_node(&mut self, node: &ruby_prism::MissingNode) {
        self.push_node(node.as_node());
        run_missing_node_rules!(node, self);
        ruby_prism::visit_missing_node(self, node);
        self.pop_node();
    }
    fn visit_module_node(&mut self, node: &ruby_prism::ModuleNode) {
        self.push_node(node.as_node());
        run_module_node_rules!(node, self);
        ruby_prism::visit_module_node(self, node);
        self.pop_node();
    }
    fn visit_multi_target_node(&mut self, node: &ruby_prism::MultiTargetNode) {
        self.push_node(node.as_node());
        run_multi_target_node_rules!(node, self);
        ruby_prism::visit_multi_target_node(self, node);
        self.pop_node();
    }
    fn visit_multi_write_node(&mut self, node: &ruby_prism::MultiWriteNode) {
        self.push_node(node.as_node());
        run_multi_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        ruby_prism::visit_multi_write_node(self, node);
        self.pop_node();
    }
    fn visit_next_node(&mut self, node: &ruby_prism::NextNode) {
        self.push_node(node.as_node());
        run_next_node_rules!(node, self);
        ruby_prism::visit_next_node(self, node);
        self.pop_node();
    }
    fn visit_nil_node(&mut self, node: &ruby_prism::NilNode) {
        self.push_node(node.as_node());
        run_nil_node_rules!(node, self);
        ruby_prism::visit_nil_node(self, node);
        self.pop_node();
    }
    fn visit_no_keywords_parameter_node(&mut self, node: &ruby_prism::NoKeywordsParameterNode) {
        self.push_node(node.as_node());
        run_no_keywords_parameter_node_rules!(node, self);
        ruby_prism::visit_no_keywords_parameter_node(self, node);
        self.pop_node();
    }
    fn visit_numbered_parameters_node(&mut self, node: &ruby_prism::NumberedParametersNode) {
        self.push_node(node.as_node());
        run_numbered_parameters_node_rules!(node, self);
        ruby_prism::visit_numbered_parameters_node(self, node);
        self.pop_node();
    }
    fn visit_numbered_reference_read_node(&mut self, node: &ruby_prism::NumberedReferenceReadNode) {
        self.push_node(node.as_node());
        run_numbered_reference_read_node_rules!(node, self);
        ruby_prism::visit_numbered_reference_read_node(self, node);
        self.pop_node();
    }
    fn visit_optional_keyword_parameter_node(&mut self, node: &ruby_prism::OptionalKeywordParameterNode) {
        self.push_node(node.as_node());
        run_optional_keyword_parameter_node_rules!(node, self);
        ruby_prism::visit_optional_keyword_parameter_node(self, node);
        self.pop_node();
    }
    fn visit_optional_parameter_node(&mut self, node: &ruby_prism::OptionalParameterNode) {
        self.push_node(node.as_node());
        run_optional_parameter_node_rules!(node, self);
        ruby_prism::visit_optional_parameter_node(self, node);
        self.pop_node();
    }
    fn visit_or_node(&mut self, node: &ruby_prism::OrNode) {
        self.push_node(node.as_node());
        run_or_node_rules!(node, self);
        ruby_prism::visit_or_node(self, node);
        self.pop_node();
    }
    fn visit_parameters_node(&mut self, node: &ruby_prism::ParametersNode) {
        self.push_node(node.as_node());
        run_parameters_node_rules!(node, self);
        ruby_prism::visit_parameters_node(self, node);
        self.pop_node();
    }
    fn visit_parentheses_node(&mut self, node: &ruby_prism::ParenthesesNode) {
        self.push_node(node.as_node());
        run_parentheses_node_rules!(node, self);
        ruby_prism::visit_parentheses_node(self, node);
        self.pop_node();
    }
    fn visit_pinned_expression_node(&mut self, node: &ruby_prism::PinnedExpressionNode) {
        self.push_node(node.as_node());
        run_pinned_expression_node_rules!(node, self);
        ruby_prism::visit_pinned_expression_node(self, node);
        self.pop_node();
    }
    fn visit_pinned_variable_node(&mut self, node: &ruby_prism::PinnedVariableNode) {
        self.push_node(node.as_node());
        run_pinned_variable_node_rules!(node, self);
        ruby_prism::visit_pinned_variable_node(self, node);
        self.pop_node();
    }
    fn visit_post_execution_node(&mut self, node: &ruby_prism::PostExecutionNode) {
        self.push_node(node.as_node());
        run_post_execution_node_rules!(node, self);
        ruby_prism::visit_post_execution_node(self, node);
        self.pop_node();
    }
    fn visit_pre_execution_node(&mut self, node: &ruby_prism::PreExecutionNode) {
        self.push_node(node.as_node());
        run_pre_execution_node_rules!(node, self);
        ruby_prism::visit_pre_execution_node(self, node);
        self.pop_node();
    }
    fn visit_program_node(&mut self, node: &ruby_prism::ProgramNode) {
        run_program_node_rules!(node, self);
        // Push/pop ancestor skipped for ProgramNode
        // self.push_node(node.as_node());
        ruby_prism::visit_program_node(self, node);
        // self.pop_node();
    }
    fn visit_range_node(&mut self, node: &ruby_prism::RangeNode) {
        self.push_node(node.as_node());
        run_range_node_rules!(node, self);
        ruby_prism::visit_range_node(self, node);
        self.pop_node();
    }
    fn visit_rational_node(&mut self, node: &ruby_prism::RationalNode) {
        self.push_node(node.as_node());
        run_rational_node_rules!(node, self);
        ruby_prism::visit_rational_node(self, node);
        self.pop_node();
    }
    fn visit_redo_node(&mut self, node: &ruby_prism::RedoNode) {
        self.push_node(node.as_node());
        run_redo_node_rules!(node, self);
        ruby_prism::visit_redo_node(self, node);
        self.pop_node();
    }
    fn visit_regular_expression_node(&mut self, node: &ruby_prism::RegularExpressionNode) {
        self.push_node(node.as_node());
        run_regular_expression_node_rules!(node, self);
        ruby_prism::visit_regular_expression_node(self, node);
        self.pop_node();
    }
    fn visit_required_keyword_parameter_node(&mut self, node: &ruby_prism::RequiredKeywordParameterNode) {
        self.push_node(node.as_node());
        run_required_keyword_parameter_node_rules!(node, self);
        ruby_prism::visit_required_keyword_parameter_node(self, node);
        self.pop_node();
    }
    fn visit_required_parameter_node(&mut self, node: &ruby_prism::RequiredParameterNode) {
        self.push_node(node.as_node());
        run_required_parameter_node_rules!(node, self);
        ruby_prism::visit_required_parameter_node(self, node);
        self.pop_node();
    }
    fn visit_rescue_modifier_node(&mut self, node: &ruby_prism::RescueModifierNode) {
        self.push_node(node.as_node());
        run_rescue_modifier_node_rules!(node, self);
        ruby_prism::visit_rescue_modifier_node(self, node);
        self.pop_node();
    }
    fn visit_rescue_node(&mut self, node: &ruby_prism::RescueNode) {
        self.push_node(node.as_node());
        run_rescue_node_rules!(node, self);
        ruby_prism::visit_rescue_node(self, node);
        self.pop_node();
    }
    fn visit_rest_parameter_node(&mut self, node: &ruby_prism::RestParameterNode) {
        self.push_node(node.as_node());
        run_rest_parameter_node_rules!(node, self);
        ruby_prism::visit_rest_parameter_node(self, node);
        self.pop_node();
    }
    fn visit_retry_node(&mut self, node: &ruby_prism::RetryNode) {
        self.push_node(node.as_node());
        run_retry_node_rules!(node, self);
        ruby_prism::visit_retry_node(self, node);
        self.pop_node();
    }
    fn visit_return_node(&mut self, node: &ruby_prism::ReturnNode) {
        self.push_node(node.as_node());
        run_return_node_rules!(node, self);
        ruby_prism::visit_return_node(self, node);
        self.pop_node();
    }
    fn visit_self_node(&mut self, node: &ruby_prism::SelfNode) {
        self.push_node(node.as_node());
        run_self_node_rules!(node, self);
        ruby_prism::visit_self_node(self, node);
        self.pop_node();
    }
    fn visit_shareable_constant_node(&mut self, node: &ruby_prism::ShareableConstantNode) {
        self.push_node(node.as_node());
        run_shareable_constant_node_rules!(node, self);
        ruby_prism::visit_shareable_constant_node(self, node);
        self.pop_node();
    }
    fn visit_singleton_class_node(&mut self, node: &ruby_prism::SingletonClassNode) {
        self.push_node(node.as_node());
        run_singleton_class_node_rules!(node, self);
        ruby_prism::visit_singleton_class_node(self, node);
        self.pop_node();
    }
    fn visit_source_encoding_node(&mut self, node: &ruby_prism::SourceEncodingNode) {
        self.push_node(node.as_node());
        run_source_encoding_node_rules!(node, self);
        ruby_prism::visit_source_encoding_node(self, node);
        self.pop_node();
    }
    fn visit_source_file_node(&mut self, node: &ruby_prism::SourceFileNode) {
        self.push_node(node.as_node());
        run_source_file_node_rules!(node, self);
        ruby_prism::visit_source_file_node(self, node);
        self.pop_node();
    }
    fn visit_source_line_node(&mut self, node: &ruby_prism::SourceLineNode) {
        self.push_node(node.as_node());
        run_source_line_node_rules!(node, self);
        ruby_prism::visit_source_line_node(self, node);
        self.pop_node();
    }
    fn visit_splat_node(&mut self, node: &ruby_prism::SplatNode) {
        self.push_node(node.as_node());
        run_splat_node_rules!(node, self);
        ruby_prism::visit_splat_node(self, node);
        self.pop_node();
    }
    fn visit_statements_node(&mut self, node: &ruby_prism::StatementsNode) {
        // Skip adding ProgramNode's StatementsNode to ancestors
        // (ProgramNode is the root and not pushed to ancestors)
        // Use current_node_id instead of parent to detect if we're inside a tracked context
        let should_track = self.semantic.current_node_id().is_some();
        should_track.then(|| self.push_node(node.as_node()));
        run_statements_node_rules!(node, self);
        ruby_prism::visit_statements_node(self, node);
        should_track.then(|| self.pop_node());
    }
    fn visit_string_node(&mut self, node: &ruby_prism::StringNode) {
        self.push_node(node.as_node());
        run_string_node_rules!(node, self);
        ruby_prism::visit_string_node(self, node);
        self.pop_node();
    }
    fn visit_super_node(&mut self, node: &ruby_prism::SuperNode) {
        self.push_node(node.as_node());
        run_super_node_rules!(node, self);
        ruby_prism::visit_super_node(self, node);
        self.pop_node();
    }
    fn visit_symbol_node(&mut self, node: &ruby_prism::SymbolNode) {
        self.push_node(node.as_node());
        run_symbol_node_rules!(node, self);
        ruby_prism::visit_symbol_node(self, node);
        self.pop_node();
    }
    fn visit_true_node(&mut self, node: &ruby_prism::TrueNode) {
        self.push_node(node.as_node());
        run_true_node_rules!(node, self);
        ruby_prism::visit_true_node(self, node);
        self.pop_node();
    }
    fn visit_undef_node(&mut self, node: &ruby_prism::UndefNode) {
        self.push_node(node.as_node());
        run_undef_node_rules!(node, self);
        ruby_prism::visit_undef_node(self, node);
        self.pop_node();
    }
    fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode) {
        self.push_node(node.as_node());
        run_unless_node_rules!(node, self);
        ruby_prism::visit_unless_node(self, node);
        self.pop_node();
    }
    fn visit_until_node(&mut self, node: &ruby_prism::UntilNode) {
        self.push_node(node.as_node());
        run_until_node_rules!(node, self);
        ruby_prism::visit_until_node(self, node);
        self.pop_node();
    }
    fn visit_when_node(&mut self, node: &ruby_prism::WhenNode) {
        self.push_node(node.as_node());
        run_when_node_rules!(node, self);
        ruby_prism::visit_when_node(self, node);
        self.pop_node();
    }
    fn visit_while_node(&mut self, node: &ruby_prism::WhileNode) {
        self.push_node(node.as_node());
        run_while_node_rules!(node, self);
        ruby_prism::visit_while_node(self, node);
        self.pop_node();
    }
    fn visit_x_string_node(&mut self, node: &ruby_prism::XStringNode) {
        self.push_node(node.as_node());
        run_x_string_node_rules!(node, self);
        ruby_prism::visit_x_string_node(self, node);
        self.pop_node();
    }
    fn visit_yield_node(&mut self, node: &ruby_prism::YieldNode) {
        self.push_node(node.as_node());
        run_yield_node_rules!(node, self);
        ruby_prism::visit_yield_node(self, node);
        self.pop_node();
    }
}
