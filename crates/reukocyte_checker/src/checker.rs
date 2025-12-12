use crate::config::Config;
use crate::custom_nodes::AssignmentNode;
use crate::diagnostic::Diagnostic;
use crate::diagnostic::Fix;
use crate::diagnostic::RawDiagnostic;
use crate::diagnostic::Severity;
use crate::locator::LineIndex;
use crate::rule::RuleId;
use ruby_prism::*;
use rustc_hash::FxHashSet;
use std::path::Path;

// Include the auto-generated rule registry macros
include!(concat!(env!("OUT_DIR"), "/rule_registry.rs"));

/// The main checker that traverses the AST and runs rules.
pub struct Checker<'rk> {
    source: &'rk [u8],
    config: &'rk Config,
    file_path: Option<&'rk Path>,
    ignored_nodes: FxHashSet<(usize, usize)>,
    line_index: LineIndex<'rk>,
    ancestor_stack: Vec<Node<'rk>>,
    raw_diagnostics: Vec<RawDiagnostic>,
}
impl<'rk> Checker<'rk> {
    /// Create a new Checker instance.
    ///
    /// `file_path` maps CLI inputs to behavior:
    /// - `Some(path)`: when checking a file or using `--stdin <FILE>` (filename known)
    /// - `None`: when reading from unnamed STDIN or when called from library APIs like `check()`
    ///
    /// The value is used for diagnostics and for rule include/exclude checks.
    pub fn new(source: &'rk [u8], config: &'rk Config, file_path: Option<&'rk Path>) -> Self {
        Self {
            source: source,
            config: config,
            file_path: file_path,
            ignored_nodes: FxHashSet::default(),
            line_index: LineIndex::from_source(source),
            ancestor_stack: Vec::new(),
            raw_diagnostics: Vec::new(),
        }
    }
    /// Run all registered AST/node-based rules (single traversal).
    #[inline]
    pub fn visit_nodes(&mut self, root: &Node<'rk>) {
        self.visit(root);
    }
    /// Run all registered line-based rules (one pass over lines).
    #[inline]
    pub fn visit_lines(&mut self) {
        let lines: Vec<_> = self.line_index.lines().iter().cloned().collect();
        for line in &lines {
            run_line_rules!(line, self);
        }
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
    /// Non-cached version that decides whether a cop should run given a `BaseCopConfig`.
    /// It will use precompiled `GlobSet`s in `BaseCopConfig` when available, otherwise
    /// fall back to matching against the pattern lists.
    #[inline]
    pub fn should_run_cop(&self, _base: &crate::config::BaseCopConfig) -> bool {
        true
    }
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
    /// Severity is passed directly from the rule (already resolved from config).
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
impl<'rk> Checker<'rk> {
    #[inline]
    fn before(&mut self, node: Node<'_>) {
        let node: Node<'rk> = unsafe { std::mem::transmute(node) };
        self.ancestor_stack.push(node);
    }
    #[inline]
    fn after(&mut self) {
        self.ancestor_stack.pop();
    }
}
impl Visit<'_> for Checker<'_> {
    fn visit_alias_global_variable_node(&mut self, node: &ruby_prism::AliasGlobalVariableNode) {
        run_alias_global_variable_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_alias_global_variable_node(self, node);
        self.after();
    }
    fn visit_alias_method_node(&mut self, node: &ruby_prism::AliasMethodNode) {
        run_alias_method_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_alias_method_node(self, node);
        self.after();
    }
    fn visit_alternation_pattern_node(&mut self, node: &ruby_prism::AlternationPatternNode) {
        run_alternation_pattern_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_alternation_pattern_node(self, node);
        self.after();
    }
    fn visit_and_node(&mut self, node: &ruby_prism::AndNode) {
        run_and_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_and_node(self, node);
        self.after();
    }
    fn visit_arguments_node(&mut self, node: &ruby_prism::ArgumentsNode) {
        run_arguments_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_arguments_node(self, node);
        self.after();
    }
    fn visit_array_node(&mut self, node: &ruby_prism::ArrayNode) {
        run_array_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_array_node(self, node);
        self.after();
    }
    fn visit_array_pattern_node(&mut self, node: &ruby_prism::ArrayPatternNode) {
        run_array_pattern_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_array_pattern_node(self, node);
        self.after();
    }
    fn visit_assoc_node(&mut self, node: &ruby_prism::AssocNode) {
        run_assoc_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_assoc_node(self, node);
        self.after();
    }
    fn visit_assoc_splat_node(&mut self, node: &ruby_prism::AssocSplatNode) {
        run_assoc_splat_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_assoc_splat_node(self, node);
        self.after();
    }
    fn visit_back_reference_read_node(&mut self, node: &ruby_prism::BackReferenceReadNode) {
        run_back_reference_read_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_back_reference_read_node(self, node);
        self.after();
    }
    fn visit_begin_node(&mut self, node: &ruby_prism::BeginNode) {
        run_begin_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_begin_node(self, node);
        self.after();
    }
    fn visit_block_argument_node(&mut self, node: &ruby_prism::BlockArgumentNode) {
        run_block_argument_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_block_argument_node(self, node);
        self.after();
    }
    fn visit_block_local_variable_node(&mut self, node: &ruby_prism::BlockLocalVariableNode) {
        run_block_local_variable_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_block_local_variable_node(self, node);
        self.after();
    }
    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode) {
        run_block_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_block_node(self, node);
        self.after();
    }
    fn visit_block_parameter_node(&mut self, node: &ruby_prism::BlockParameterNode) {
        run_block_parameter_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_block_parameter_node(self, node);
        self.after();
    }
    fn visit_block_parameters_node(&mut self, node: &ruby_prism::BlockParametersNode) {
        run_block_parameters_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_block_parameters_node(self, node);
        self.after();
    }
    fn visit_break_node(&mut self, node: &ruby_prism::BreakNode) {
        run_break_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_break_node(self, node);
        self.after();
    }
    fn visit_call_and_write_node(&mut self, node: &ruby_prism::CallAndWriteNode) {
        run_call_and_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_call_and_write_node(self, node);
        self.after();
    }
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode) {
        run_call_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_call_node(self, node);
        self.after();
    }
    fn visit_call_operator_write_node(&mut self, node: &ruby_prism::CallOperatorWriteNode) {
        run_call_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_call_operator_write_node(self, node);
        self.after();
    }
    fn visit_call_or_write_node(&mut self, node: &ruby_prism::CallOrWriteNode) {
        run_call_or_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_call_or_write_node(self, node);
        self.after();
    }
    fn visit_call_target_node(&mut self, node: &ruby_prism::CallTargetNode) {
        run_call_target_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_call_target_node(self, node);
        self.after();
    }
    fn visit_capture_pattern_node(&mut self, node: &ruby_prism::CapturePatternNode) {
        run_capture_pattern_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_capture_pattern_node(self, node);
        self.after();
    }
    fn visit_case_match_node(&mut self, node: &ruby_prism::CaseMatchNode) {
        run_case_match_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_case_match_node(self, node);
        self.after();
    }
    fn visit_case_node(&mut self, node: &ruby_prism::CaseNode) {
        run_case_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_case_node(self, node);
        self.after();
    }
    fn visit_class_node(&mut self, node: &ruby_prism::ClassNode) {
        run_class_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_class_node(self, node);
        self.after();
    }
    fn visit_class_variable_and_write_node(&mut self, node: &ruby_prism::ClassVariableAndWriteNode) {
        run_class_variable_and_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_class_variable_and_write_node(self, node);
        self.after();
    }
    fn visit_class_variable_operator_write_node(&mut self, node: &ruby_prism::ClassVariableOperatorWriteNode) {
        run_class_variable_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_class_variable_operator_write_node(self, node);
        self.after();
    }
    fn visit_class_variable_or_write_node(&mut self, node: &ruby_prism::ClassVariableOrWriteNode) {
        run_class_variable_or_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_class_variable_or_write_node(self, node);
        self.after();
    }
    fn visit_class_variable_read_node(&mut self, node: &ruby_prism::ClassVariableReadNode) {
        run_class_variable_read_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_class_variable_read_node(self, node);
        self.after();
    }
    fn visit_class_variable_target_node(&mut self, node: &ruby_prism::ClassVariableTargetNode) {
        run_class_variable_target_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_class_variable_target_node(self, node);
        self.after();
    }
    fn visit_class_variable_write_node(&mut self, node: &ruby_prism::ClassVariableWriteNode) {
        run_class_variable_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_class_variable_write_node(self, node);
        self.after();
    }
    fn visit_constant_and_write_node(&mut self, node: &ruby_prism::ConstantAndWriteNode) {
        run_constant_and_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_constant_and_write_node(self, node);
        self.after();
    }
    fn visit_constant_operator_write_node(&mut self, node: &ruby_prism::ConstantOperatorWriteNode) {
        run_constant_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_constant_operator_write_node(self, node);
        self.after();
    }
    fn visit_constant_or_write_node(&mut self, node: &ruby_prism::ConstantOrWriteNode) {
        run_constant_or_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_constant_or_write_node(self, node);
        self.after();
    }
    fn visit_constant_path_and_write_node(&mut self, node: &ruby_prism::ConstantPathAndWriteNode) {
        run_constant_path_and_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_constant_path_and_write_node(self, node);
        self.after();
    }
    fn visit_constant_path_node(&mut self, node: &ruby_prism::ConstantPathNode) {
        run_constant_path_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_constant_path_node(self, node);
        self.after();
    }
    fn visit_constant_path_operator_write_node(&mut self, node: &ruby_prism::ConstantPathOperatorWriteNode) {
        run_constant_path_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_constant_path_operator_write_node(self, node);
        self.after();
    }
    fn visit_constant_path_or_write_node(&mut self, node: &ruby_prism::ConstantPathOrWriteNode) {
        run_constant_path_or_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_constant_path_or_write_node(self, node);
        self.after();
    }
    fn visit_constant_path_target_node(&mut self, node: &ruby_prism::ConstantPathTargetNode) {
        run_constant_path_target_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_constant_path_target_node(self, node);
        self.after();
    }
    fn visit_constant_path_write_node(&mut self, node: &ruby_prism::ConstantPathWriteNode) {
        run_constant_path_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_constant_path_write_node(self, node);
        self.after();
    }
    fn visit_constant_read_node(&mut self, node: &ruby_prism::ConstantReadNode) {
        run_constant_read_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_constant_read_node(self, node);
        self.after();
    }
    fn visit_constant_target_node(&mut self, node: &ruby_prism::ConstantTargetNode) {
        run_constant_target_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_constant_target_node(self, node);
        self.after();
    }
    fn visit_constant_write_node(&mut self, node: &ruby_prism::ConstantWriteNode) {
        run_constant_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_constant_write_node(self, node);
        self.after();
    }
    fn visit_def_node(&mut self, node: &ruby_prism::DefNode) {
        run_def_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_def_node(self, node);
        self.after();
    }
    fn visit_defined_node(&mut self, node: &ruby_prism::DefinedNode) {
        run_defined_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_defined_node(self, node);
        self.after();
    }
    fn visit_else_node(&mut self, node: &ruby_prism::ElseNode) {
        run_else_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_else_node(self, node);
        self.after();
    }
    fn visit_embedded_statements_node(&mut self, node: &ruby_prism::EmbeddedStatementsNode) {
        run_embedded_statements_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_embedded_statements_node(self, node);
        self.after();
    }
    fn visit_embedded_variable_node(&mut self, node: &ruby_prism::EmbeddedVariableNode) {
        run_embedded_variable_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_embedded_variable_node(self, node);
        self.after();
    }
    fn visit_ensure_node(&mut self, node: &ruby_prism::EnsureNode) {
        run_ensure_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_ensure_node(self, node);
        self.after();
    }
    fn visit_false_node(&mut self, node: &ruby_prism::FalseNode) {
        run_false_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_false_node(self, node);
        self.after();
    }
    fn visit_find_pattern_node(&mut self, node: &ruby_prism::FindPatternNode) {
        run_find_pattern_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_find_pattern_node(self, node);
        self.after();
    }
    fn visit_flip_flop_node(&mut self, node: &ruby_prism::FlipFlopNode) {
        run_flip_flop_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_flip_flop_node(self, node);
        self.after();
    }
    fn visit_float_node(&mut self, node: &ruby_prism::FloatNode) {
        run_float_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_float_node(self, node);
        self.after();
    }
    fn visit_for_node(&mut self, node: &ruby_prism::ForNode) {
        run_for_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_for_node(self, node);
        self.after();
    }
    fn visit_forwarding_arguments_node(&mut self, node: &ruby_prism::ForwardingArgumentsNode) {
        run_forwarding_arguments_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_forwarding_arguments_node(self, node);
        self.after();
    }
    fn visit_forwarding_parameter_node(&mut self, node: &ruby_prism::ForwardingParameterNode) {
        run_forwarding_parameter_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_forwarding_parameter_node(self, node);
        self.after();
    }
    fn visit_forwarding_super_node(&mut self, node: &ruby_prism::ForwardingSuperNode) {
        run_forwarding_super_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_forwarding_super_node(self, node);
        self.after();
    }
    fn visit_global_variable_and_write_node(&mut self, node: &ruby_prism::GlobalVariableAndWriteNode) {
        run_global_variable_and_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_global_variable_and_write_node(self, node);
        self.after();
    }
    fn visit_global_variable_operator_write_node(&mut self, node: &ruby_prism::GlobalVariableOperatorWriteNode) {
        run_global_variable_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_global_variable_operator_write_node(self, node);
        self.after();
    }
    fn visit_global_variable_or_write_node(&mut self, node: &ruby_prism::GlobalVariableOrWriteNode) {
        run_global_variable_or_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_global_variable_or_write_node(self, node);
        self.after();
    }
    fn visit_global_variable_read_node(&mut self, node: &ruby_prism::GlobalVariableReadNode) {
        run_global_variable_read_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_global_variable_read_node(self, node);
        self.after();
    }
    fn visit_global_variable_target_node(&mut self, node: &ruby_prism::GlobalVariableTargetNode) {
        run_global_variable_target_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_global_variable_target_node(self, node);
        self.after();
    }
    fn visit_global_variable_write_node(&mut self, node: &ruby_prism::GlobalVariableWriteNode) {
        run_global_variable_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_global_variable_write_node(self, node);
        self.after();
    }
    fn visit_hash_node(&mut self, node: &ruby_prism::HashNode) {
        run_hash_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_hash_node(self, node);
        self.after();
    }
    fn visit_hash_pattern_node(&mut self, node: &ruby_prism::HashPatternNode) {
        run_hash_pattern_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_hash_pattern_node(self, node);
        self.after();
    }
    fn visit_if_node(&mut self, node: &ruby_prism::IfNode) {
        run_if_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_if_node(self, node);
        self.after();
    }
    fn visit_imaginary_node(&mut self, node: &ruby_prism::ImaginaryNode) {
        run_imaginary_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_imaginary_node(self, node);
        self.after();
    }
    fn visit_implicit_node(&mut self, node: &ruby_prism::ImplicitNode) {
        run_implicit_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_implicit_node(self, node);
        self.after();
    }
    fn visit_implicit_rest_node(&mut self, node: &ruby_prism::ImplicitRestNode) {
        run_implicit_rest_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_implicit_rest_node(self, node);
        self.after();
    }
    fn visit_in_node(&mut self, node: &ruby_prism::InNode) {
        run_in_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_in_node(self, node);
        self.after();
    }
    fn visit_index_and_write_node(&mut self, node: &ruby_prism::IndexAndWriteNode) {
        run_index_and_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_index_and_write_node(self, node);
        self.after();
    }
    fn visit_index_operator_write_node(&mut self, node: &ruby_prism::IndexOperatorWriteNode) {
        run_index_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_index_operator_write_node(self, node);
        self.after();
    }
    fn visit_index_or_write_node(&mut self, node: &ruby_prism::IndexOrWriteNode) {
        run_index_or_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_index_or_write_node(self, node);
        self.after();
    }
    fn visit_index_target_node(&mut self, node: &ruby_prism::IndexTargetNode) {
        run_index_target_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_index_target_node(self, node);
        self.after();
    }
    fn visit_instance_variable_and_write_node(&mut self, node: &ruby_prism::InstanceVariableAndWriteNode) {
        run_instance_variable_and_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_instance_variable_and_write_node(self, node);
        self.after();
    }
    fn visit_instance_variable_operator_write_node(&mut self, node: &ruby_prism::InstanceVariableOperatorWriteNode) {
        run_instance_variable_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_instance_variable_operator_write_node(self, node);
        self.after();
    }
    fn visit_instance_variable_or_write_node(&mut self, node: &ruby_prism::InstanceVariableOrWriteNode) {
        run_instance_variable_or_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_instance_variable_or_write_node(self, node);
        self.after();
    }
    fn visit_instance_variable_read_node(&mut self, node: &ruby_prism::InstanceVariableReadNode) {
        run_instance_variable_read_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_instance_variable_read_node(self, node);
        self.after();
    }
    fn visit_instance_variable_target_node(&mut self, node: &ruby_prism::InstanceVariableTargetNode) {
        run_instance_variable_target_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_instance_variable_target_node(self, node);
        self.after();
    }
    fn visit_instance_variable_write_node(&mut self, node: &ruby_prism::InstanceVariableWriteNode) {
        run_instance_variable_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_instance_variable_write_node(self, node);
        self.after();
    }
    fn visit_integer_node(&mut self, node: &ruby_prism::IntegerNode) {
        run_integer_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_integer_node(self, node);
        self.after();
    }
    fn visit_interpolated_match_last_line_node(&mut self, node: &ruby_prism::InterpolatedMatchLastLineNode) {
        run_interpolated_match_last_line_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_interpolated_match_last_line_node(self, node);
        self.after();
    }
    fn visit_interpolated_regular_expression_node(&mut self, node: &ruby_prism::InterpolatedRegularExpressionNode) {
        run_interpolated_regular_expression_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_interpolated_regular_expression_node(self, node);
        self.after();
    }
    fn visit_interpolated_string_node(&mut self, node: &ruby_prism::InterpolatedStringNode) {
        run_interpolated_string_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_interpolated_string_node(self, node);
        self.after();
    }
    fn visit_interpolated_symbol_node(&mut self, node: &ruby_prism::InterpolatedSymbolNode) {
        run_interpolated_symbol_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_interpolated_symbol_node(self, node);
        self.after();
    }
    fn visit_interpolated_x_string_node(&mut self, node: &ruby_prism::InterpolatedXStringNode) {
        run_interpolated_x_string_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_interpolated_x_string_node(self, node);
        self.after();
    }
    fn visit_it_local_variable_read_node(&mut self, node: &ruby_prism::ItLocalVariableReadNode) {
        run_it_local_variable_read_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_it_local_variable_read_node(self, node);
        self.after();
    }
    fn visit_it_parameters_node(&mut self, node: &ruby_prism::ItParametersNode) {
        run_it_parameters_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_it_parameters_node(self, node);
        self.after();
    }
    fn visit_keyword_hash_node(&mut self, node: &ruby_prism::KeywordHashNode) {
        run_keyword_hash_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_keyword_hash_node(self, node);
        self.after();
    }
    fn visit_keyword_rest_parameter_node(&mut self, node: &ruby_prism::KeywordRestParameterNode) {
        run_keyword_rest_parameter_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_keyword_rest_parameter_node(self, node);
        self.after();
    }
    fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode) {
        run_lambda_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_lambda_node(self, node);
        self.after();
    }
    fn visit_local_variable_and_write_node(&mut self, node: &ruby_prism::LocalVariableAndWriteNode) {
        run_local_variable_and_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_local_variable_and_write_node(self, node);
        self.after();
    }
    fn visit_local_variable_operator_write_node(&mut self, node: &ruby_prism::LocalVariableOperatorWriteNode) {
        run_local_variable_operator_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_local_variable_operator_write_node(self, node);
        self.after();
    }
    fn visit_local_variable_or_write_node(&mut self, node: &ruby_prism::LocalVariableOrWriteNode) {
        run_local_variable_or_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_local_variable_or_write_node(self, node);
        self.after();
    }
    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode) {
        run_local_variable_read_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_local_variable_read_node(self, node);
        self.after();
    }
    fn visit_local_variable_target_node(&mut self, node: &ruby_prism::LocalVariableTargetNode) {
        run_local_variable_target_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_local_variable_target_node(self, node);
        self.after();
    }
    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode) {
        run_local_variable_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_local_variable_write_node(self, node);
        self.after();
    }
    fn visit_match_last_line_node(&mut self, node: &ruby_prism::MatchLastLineNode) {
        run_match_last_line_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_match_last_line_node(self, node);
        self.after();
    }
    fn visit_match_predicate_node(&mut self, node: &ruby_prism::MatchPredicateNode) {
        run_match_predicate_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_match_predicate_node(self, node);
        self.after();
    }
    fn visit_match_required_node(&mut self, node: &ruby_prism::MatchRequiredNode) {
        run_match_required_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_match_required_node(self, node);
        self.after();
    }
    fn visit_match_write_node(&mut self, node: &ruby_prism::MatchWriteNode) {
        run_match_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);

        self.before(node.as_node());
        ruby_prism::visit_match_write_node(self, node);
        self.after();
    }
    fn visit_missing_node(&mut self, node: &ruby_prism::MissingNode) {
        run_missing_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_missing_node(self, node);
        self.after();
    }
    fn visit_module_node(&mut self, node: &ruby_prism::ModuleNode) {
        run_module_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_module_node(self, node);
        self.after();
    }
    fn visit_multi_target_node(&mut self, node: &ruby_prism::MultiTargetNode) {
        run_multi_target_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_multi_target_node(self, node);
        self.after();
    }
    fn visit_multi_write_node(&mut self, node: &ruby_prism::MultiWriteNode) {
        run_multi_write_node_rules!(node, self);
        run_assignment_node_rules!(&AssignmentNode::from(node), self);
        self.before(node.as_node());
        ruby_prism::visit_multi_write_node(self, node);
        self.after();
    }
    fn visit_next_node(&mut self, node: &ruby_prism::NextNode) {
        run_next_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_next_node(self, node);
        self.after();
    }
    fn visit_nil_node(&mut self, node: &ruby_prism::NilNode) {
        run_nil_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_nil_node(self, node);
        self.after();
    }
    fn visit_no_keywords_parameter_node(&mut self, node: &ruby_prism::NoKeywordsParameterNode) {
        run_no_keywords_parameter_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_no_keywords_parameter_node(self, node);
        self.after();
    }
    fn visit_numbered_parameters_node(&mut self, node: &ruby_prism::NumberedParametersNode) {
        run_numbered_parameters_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_numbered_parameters_node(self, node);
        self.after();
    }
    fn visit_numbered_reference_read_node(&mut self, node: &ruby_prism::NumberedReferenceReadNode) {
        run_numbered_reference_read_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_numbered_reference_read_node(self, node);
        self.after();
    }
    fn visit_optional_keyword_parameter_node(&mut self, node: &ruby_prism::OptionalKeywordParameterNode) {
        run_optional_keyword_parameter_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_optional_keyword_parameter_node(self, node);
        self.after();
    }
    fn visit_optional_parameter_node(&mut self, node: &ruby_prism::OptionalParameterNode) {
        run_optional_parameter_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_optional_parameter_node(self, node);
        self.after();
    }
    fn visit_or_node(&mut self, node: &ruby_prism::OrNode) {
        run_or_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_or_node(self, node);
        self.after();
    }
    fn visit_parameters_node(&mut self, node: &ruby_prism::ParametersNode) {
        run_parameters_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_parameters_node(self, node);
        self.after();
    }
    fn visit_parentheses_node(&mut self, node: &ruby_prism::ParenthesesNode) {
        run_parentheses_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_parentheses_node(self, node);
        self.after();
    }
    fn visit_pinned_expression_node(&mut self, node: &ruby_prism::PinnedExpressionNode) {
        run_pinned_expression_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_pinned_expression_node(self, node);
        self.after();
    }
    fn visit_pinned_variable_node(&mut self, node: &ruby_prism::PinnedVariableNode) {
        run_pinned_variable_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_pinned_variable_node(self, node);
        self.after();
    }
    fn visit_post_execution_node(&mut self, node: &ruby_prism::PostExecutionNode) {
        run_post_execution_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_post_execution_node(self, node);
        self.after();
    }
    fn visit_pre_execution_node(&mut self, node: &ruby_prism::PreExecutionNode) {
        run_pre_execution_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_pre_execution_node(self, node);
        self.after();
    }
    fn visit_program_node(&mut self, node: &ruby_prism::ProgramNode) {
        run_program_node_rules!(node, self);
        // Push/pop ancestor skipped for ProgramNode
        //
        self.before(node.as_node());
        ruby_prism::visit_program_node(self, node);
        self.after();
        //
    }
    fn visit_range_node(&mut self, node: &ruby_prism::RangeNode) {
        run_range_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_range_node(self, node);
        self.after();
    }
    fn visit_rational_node(&mut self, node: &ruby_prism::RationalNode) {
        run_rational_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_rational_node(self, node);
        self.after();
    }
    fn visit_redo_node(&mut self, node: &ruby_prism::RedoNode) {
        run_redo_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_redo_node(self, node);
        self.after();
    }
    fn visit_regular_expression_node(&mut self, node: &ruby_prism::RegularExpressionNode) {
        run_regular_expression_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_regular_expression_node(self, node);
        self.after();
    }
    fn visit_required_keyword_parameter_node(&mut self, node: &ruby_prism::RequiredKeywordParameterNode) {
        run_required_keyword_parameter_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_required_keyword_parameter_node(self, node);
        self.after();
    }
    fn visit_required_parameter_node(&mut self, node: &ruby_prism::RequiredParameterNode) {
        run_required_parameter_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_required_parameter_node(self, node);
        self.after();
    }
    fn visit_rescue_modifier_node(&mut self, node: &ruby_prism::RescueModifierNode) {
        run_rescue_modifier_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_rescue_modifier_node(self, node);
        self.after();
    }
    fn visit_rescue_node(&mut self, node: &ruby_prism::RescueNode) {
        run_rescue_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_rescue_node(self, node);
        self.after();
    }
    fn visit_rest_parameter_node(&mut self, node: &ruby_prism::RestParameterNode) {
        run_rest_parameter_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_rest_parameter_node(self, node);
        self.after();
    }
    fn visit_retry_node(&mut self, node: &ruby_prism::RetryNode) {
        run_retry_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_retry_node(self, node);
        self.after();
    }
    fn visit_return_node(&mut self, node: &ruby_prism::ReturnNode) {
        run_return_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_return_node(self, node);
        self.after();
    }
    fn visit_self_node(&mut self, node: &ruby_prism::SelfNode) {
        run_self_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_self_node(self, node);
        self.after();
    }
    fn visit_shareable_constant_node(&mut self, node: &ruby_prism::ShareableConstantNode) {
        run_shareable_constant_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_shareable_constant_node(self, node);
        self.after();
    }
    fn visit_singleton_class_node(&mut self, node: &ruby_prism::SingletonClassNode) {
        //run_singleton_class_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_singleton_class_node(self, node);
        self.after();
    }
    fn visit_source_encoding_node(&mut self, node: &ruby_prism::SourceEncodingNode) {
        run_source_encoding_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_source_encoding_node(self, node);
        self.after();
    }
    fn visit_source_file_node(&mut self, node: &ruby_prism::SourceFileNode) {
        run_source_file_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_source_file_node(self, node);
        self.after();
    }
    fn visit_source_line_node(&mut self, node: &ruby_prism::SourceLineNode) {
        run_source_line_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_source_line_node(self, node);
        self.after();
    }
    fn visit_splat_node(&mut self, node: &ruby_prism::SplatNode) {
        run_splat_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_splat_node(self, node);
        self.after();
    }
    fn visit_statements_node(&mut self, node: &ruby_prism::StatementsNode) {
        // Skip adding ProgramNode's StatementsNode to ancestors
        // (ProgramNode is the root and not pushed to ancestors)
        // Use current_node_id instead of parent to detect if we're inside a tracked context
        // let should_track = self.semantic.current_node_id().is_some();
        // should_track.then(|| self.push_node(node.as_node()));
        run_statements_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_statements_node(self, node);
        self.after();
        // should_track.then(|| self.pop_node());
    }
    fn visit_string_node(&mut self, node: &ruby_prism::StringNode) {
        run_string_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_string_node(self, node);
        self.after();
    }
    fn visit_super_node(&mut self, node: &ruby_prism::SuperNode) {
        run_super_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_super_node(self, node);
        self.after();
    }
    fn visit_symbol_node(&mut self, node: &ruby_prism::SymbolNode) {
        run_symbol_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_symbol_node(self, node);
        self.after();
    }
    fn visit_true_node(&mut self, node: &ruby_prism::TrueNode) {
        run_true_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_true_node(self, node);
        self.after();
    }
    fn visit_undef_node(&mut self, node: &ruby_prism::UndefNode) {
        run_undef_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_undef_node(self, node);
        self.after();
    }
    fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode) {
        run_unless_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_unless_node(self, node);
        self.after();
    }
    fn visit_until_node(&mut self, node: &ruby_prism::UntilNode) {
        run_until_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_until_node(self, node);
        self.after();
    }
    fn visit_when_node(&mut self, node: &ruby_prism::WhenNode) {
        run_when_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_when_node(self, node);
        self.after();
    }
    fn visit_while_node(&mut self, node: &ruby_prism::WhileNode) {
        run_while_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_while_node(self, node);
        self.after();
    }
    fn visit_x_string_node(&mut self, node: &ruby_prism::XStringNode) {
        run_x_string_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_x_string_node(self, node);
        self.after();
    }
    fn visit_yield_node(&mut self, node: &ruby_prism::YieldNode) {
        run_yield_node_rules!(node, self);
        self.before(node.as_node());
        ruby_prism::visit_yield_node(self, node);
        self.after();
    }
}
