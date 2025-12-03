use crate::analyze;
use crate::diagnostic::RawDiagnostic;
use crate::locator::LineIndex;
use crate::rule::RuleId;
use crate::{Diagnostic, Fix, Severity};
use ruby_prism::Visit;

/// The main checker that traverses the AST and runs rules.
pub struct Checker<'rk> {
    source: &'rk [u8],
    raw_diagnostics: Vec<RawDiagnostic>,
}

impl<'rk> Checker<'rk> {
    pub fn new(source: &'rk [u8]) -> Self {
        Self {
            source,
            raw_diagnostics: Vec::new(),
        }
    }
    pub fn source(&self) -> &[u8] {
        self.source
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

        // Build line index once
        let line_index = LineIndex::from_source(self.source);

        // Collect offsets for batch resolution
        let offsets: Vec<(usize, usize)> = self.raw_diagnostics.iter().map(|d| (d.start, d.end)).collect();

        // Batch resolve all line/column pairs
        let resolved = line_index.batch_line_column(&offsets);

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
    fn visit_alias_global_variable_node(&mut self, node: &ruby_prism::AliasGlobalVariableNode) {
        ruby_prism::visit_alias_global_variable_node(self, node);
        analyze::alias_global_variable_node(node, self);
    }
    fn visit_alias_method_node(&mut self, node: &ruby_prism::AliasMethodNode) {
        ruby_prism::visit_alias_method_node(self, node);
        analyze::alias_method_node(node, self);
    }
    fn visit_alternation_pattern_node(&mut self, node: &ruby_prism::AlternationPatternNode) {
        ruby_prism::visit_alternation_pattern_node(self, node);
        analyze::alternation_pattern_node(node, self);
    }
    fn visit_and_node(&mut self, node: &ruby_prism::AndNode) {
        ruby_prism::visit_and_node(self, node);
        analyze::and_node(node, self);
    }
    fn visit_arguments_node(&mut self, node: &ruby_prism::ArgumentsNode) {
        ruby_prism::visit_arguments_node(self, node);
        analyze::arguments_node(node, self);
    }
    fn visit_array_node(&mut self, node: &ruby_prism::ArrayNode) {
        ruby_prism::visit_array_node(self, node);
        analyze::array_node(node, self);
    }
    fn visit_array_pattern_node(&mut self, node: &ruby_prism::ArrayPatternNode) {
        ruby_prism::visit_array_pattern_node(self, node);
        analyze::array_pattern_node(node, self);
    }
    fn visit_assoc_node(&mut self, node: &ruby_prism::AssocNode) {
        ruby_prism::visit_assoc_node(self, node);
        analyze::assoc_node(node, self);
    }
    fn visit_assoc_splat_node(&mut self, node: &ruby_prism::AssocSplatNode) {
        ruby_prism::visit_assoc_splat_node(self, node);
        analyze::assoc_splat_node(node, self);
    }
    fn visit_back_reference_read_node(&mut self, node: &ruby_prism::BackReferenceReadNode) {
        ruby_prism::visit_back_reference_read_node(self, node);
        analyze::back_reference_read_node(node, self);
    }
    fn visit_begin_node(&mut self, node: &ruby_prism::BeginNode) {
        ruby_prism::visit_begin_node(self, node);
        analyze::begin_node(node, self);
    }
    fn visit_block_argument_node(&mut self, node: &ruby_prism::BlockArgumentNode) {
        ruby_prism::visit_block_argument_node(self, node);
        analyze::block_argument_node(node, self);
    }
    fn visit_block_local_variable_node(&mut self, node: &ruby_prism::BlockLocalVariableNode) {
        ruby_prism::visit_block_local_variable_node(self, node);
        analyze::block_local_variable_node(node, self);
    }
    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode) {
        ruby_prism::visit_block_node(self, node);
        analyze::block_node(node, self);
    }
    fn visit_block_parameter_node(&mut self, node: &ruby_prism::BlockParameterNode) {
        ruby_prism::visit_block_parameter_node(self, node);
        analyze::block_parameter_node(node, self);
    }
    fn visit_block_parameters_node(&mut self, node: &ruby_prism::BlockParametersNode) {
        ruby_prism::visit_block_parameters_node(self, node);
        analyze::block_parameters_node(node, self);
    }
    fn visit_break_node(&mut self, node: &ruby_prism::BreakNode) {
        ruby_prism::visit_break_node(self, node);
        analyze::break_node(node, self);
    }
    fn visit_call_and_write_node(&mut self, node: &ruby_prism::CallAndWriteNode) {
        ruby_prism::visit_call_and_write_node(self, node);
        analyze::call_and_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode) {
        ruby_prism::visit_call_node(self, node);
        analyze::call_node(node, self);
    }
    fn visit_call_operator_write_node(&mut self, node: &ruby_prism::CallOperatorWriteNode) {
        ruby_prism::visit_call_operator_write_node(self, node);
        analyze::call_operator_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_call_or_write_node(&mut self, node: &ruby_prism::CallOrWriteNode) {
        ruby_prism::visit_call_or_write_node(self, node);
        analyze::call_or_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_call_target_node(&mut self, node: &ruby_prism::CallTargetNode) {
        ruby_prism::visit_call_target_node(self, node);
        analyze::call_target_node(node, self);
    }
    fn visit_capture_pattern_node(&mut self, node: &ruby_prism::CapturePatternNode) {
        ruby_prism::visit_capture_pattern_node(self, node);
        analyze::capture_pattern_node(node, self);
    }
    fn visit_case_match_node(&mut self, node: &ruby_prism::CaseMatchNode) {
        ruby_prism::visit_case_match_node(self, node);
        analyze::case_match_node(node, self);
    }
    fn visit_case_node(&mut self, node: &ruby_prism::CaseNode) {
        ruby_prism::visit_case_node(self, node);
        analyze::case_node(node, self);
    }
    fn visit_class_node(&mut self, node: &ruby_prism::ClassNode) {
        ruby_prism::visit_class_node(self, node);
        analyze::class_node(node, self);
    }
    fn visit_class_variable_and_write_node(&mut self, node: &ruby_prism::ClassVariableAndWriteNode) {
        ruby_prism::visit_class_variable_and_write_node(self, node);
        analyze::class_variable_and_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_class_variable_operator_write_node(&mut self, node: &ruby_prism::ClassVariableOperatorWriteNode) {
        ruby_prism::visit_class_variable_operator_write_node(self, node);
        analyze::class_variable_operator_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_class_variable_or_write_node(&mut self, node: &ruby_prism::ClassVariableOrWriteNode) {
        ruby_prism::visit_class_variable_or_write_node(self, node);
        analyze::class_variable_or_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_class_variable_read_node(&mut self, node: &ruby_prism::ClassVariableReadNode) {
        ruby_prism::visit_class_variable_read_node(self, node);
        analyze::class_variable_read_node(node, self);
    }
    fn visit_class_variable_target_node(&mut self, node: &ruby_prism::ClassVariableTargetNode) {
        ruby_prism::visit_class_variable_target_node(self, node);
        analyze::class_variable_target_node(node, self);
    }
    fn visit_class_variable_write_node(&mut self, node: &ruby_prism::ClassVariableWriteNode) {
        ruby_prism::visit_class_variable_write_node(self, node);
        analyze::class_variable_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_constant_and_write_node(&mut self, node: &ruby_prism::ConstantAndWriteNode) {
        ruby_prism::visit_constant_and_write_node(self, node);
        analyze::constant_and_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_constant_operator_write_node(&mut self, node: &ruby_prism::ConstantOperatorWriteNode) {
        ruby_prism::visit_constant_operator_write_node(self, node);
        analyze::constant_operator_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_constant_or_write_node(&mut self, node: &ruby_prism::ConstantOrWriteNode) {
        ruby_prism::visit_constant_or_write_node(self, node);
        analyze::constant_or_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_constant_path_and_write_node(&mut self, node: &ruby_prism::ConstantPathAndWriteNode) {
        ruby_prism::visit_constant_path_and_write_node(self, node);
        analyze::constant_path_and_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_constant_path_node(&mut self, node: &ruby_prism::ConstantPathNode) {
        ruby_prism::visit_constant_path_node(self, node);
        analyze::constant_path_node(node, self);
    }
    fn visit_constant_path_operator_write_node(&mut self, node: &ruby_prism::ConstantPathOperatorWriteNode) {
        ruby_prism::visit_constant_path_operator_write_node(self, node);
        analyze::constant_path_operator_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_constant_path_or_write_node(&mut self, node: &ruby_prism::ConstantPathOrWriteNode) {
        ruby_prism::visit_constant_path_or_write_node(self, node);
        analyze::constant_path_or_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_constant_path_target_node(&mut self, node: &ruby_prism::ConstantPathTargetNode) {
        ruby_prism::visit_constant_path_target_node(self, node);
        analyze::constant_path_target_node(node, self);
    }
    fn visit_constant_path_write_node(&mut self, node: &ruby_prism::ConstantPathWriteNode) {
        ruby_prism::visit_constant_path_write_node(self, node);
        analyze::constant_path_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_constant_read_node(&mut self, node: &ruby_prism::ConstantReadNode) {
        ruby_prism::visit_constant_read_node(self, node);
        analyze::constant_read_node(node, self);
    }
    fn visit_constant_target_node(&mut self, node: &ruby_prism::ConstantTargetNode) {
        ruby_prism::visit_constant_target_node(self, node);
        analyze::constant_target_node(node, self);
    }
    fn visit_constant_write_node(&mut self, node: &ruby_prism::ConstantWriteNode) {
        ruby_prism::visit_constant_write_node(self, node);
        analyze::constant_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_def_node(&mut self, node: &ruby_prism::DefNode) {
        ruby_prism::visit_def_node(self, node);
        analyze::def_node(node, self);
    }
    fn visit_defined_node(&mut self, node: &ruby_prism::DefinedNode) {
        ruby_prism::visit_defined_node(self, node);
        analyze::defined_node(node, self);
    }
    fn visit_else_node(&mut self, node: &ruby_prism::ElseNode) {
        ruby_prism::visit_else_node(self, node);
        analyze::else_node(node, self);
    }
    fn visit_embedded_statements_node(&mut self, node: &ruby_prism::EmbeddedStatementsNode) {
        ruby_prism::visit_embedded_statements_node(self, node);
        analyze::embedded_statements_node(node, self);
    }
    fn visit_embedded_variable_node(&mut self, node: &ruby_prism::EmbeddedVariableNode) {
        ruby_prism::visit_embedded_variable_node(self, node);
        analyze::embedded_variable_node(node, self);
    }
    fn visit_ensure_node(&mut self, node: &ruby_prism::EnsureNode) {
        ruby_prism::visit_ensure_node(self, node);
        analyze::ensure_node(node, self);
    }
    fn visit_false_node(&mut self, node: &ruby_prism::FalseNode) {
        ruby_prism::visit_false_node(self, node);
        analyze::false_node(node, self);
    }
    fn visit_find_pattern_node(&mut self, node: &ruby_prism::FindPatternNode) {
        ruby_prism::visit_find_pattern_node(self, node);
        analyze::find_pattern_node(node, self);
    }
    fn visit_flip_flop_node(&mut self, node: &ruby_prism::FlipFlopNode) {
        ruby_prism::visit_flip_flop_node(self, node);
        analyze::flip_flop_node(node, self);
    }
    fn visit_float_node(&mut self, node: &ruby_prism::FloatNode) {
        ruby_prism::visit_float_node(self, node);
        analyze::float_node(node, self);
    }
    fn visit_for_node(&mut self, node: &ruby_prism::ForNode) {
        ruby_prism::visit_for_node(self, node);
        analyze::for_node(node, self);
    }
    fn visit_forwarding_arguments_node(&mut self, node: &ruby_prism::ForwardingArgumentsNode) {
        ruby_prism::visit_forwarding_arguments_node(self, node);
        analyze::forwarding_arguments_node(node, self);
    }
    fn visit_forwarding_parameter_node(&mut self, node: &ruby_prism::ForwardingParameterNode) {
        ruby_prism::visit_forwarding_parameter_node(self, node);
        analyze::forwarding_parameter_node(node, self);
    }
    fn visit_forwarding_super_node(&mut self, node: &ruby_prism::ForwardingSuperNode) {
        ruby_prism::visit_forwarding_super_node(self, node);
        analyze::forwarding_super_node(node, self);
    }
    fn visit_global_variable_and_write_node(&mut self, node: &ruby_prism::GlobalVariableAndWriteNode) {
        ruby_prism::visit_global_variable_and_write_node(self, node);
        analyze::global_variable_and_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_global_variable_operator_write_node(&mut self, node: &ruby_prism::GlobalVariableOperatorWriteNode) {
        ruby_prism::visit_global_variable_operator_write_node(self, node);
        analyze::global_variable_operator_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_global_variable_or_write_node(&mut self, node: &ruby_prism::GlobalVariableOrWriteNode) {
        ruby_prism::visit_global_variable_or_write_node(self, node);
        analyze::global_variable_or_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_global_variable_read_node(&mut self, node: &ruby_prism::GlobalVariableReadNode) {
        ruby_prism::visit_global_variable_read_node(self, node);
        analyze::global_variable_read_node(node, self);
    }
    fn visit_global_variable_target_node(&mut self, node: &ruby_prism::GlobalVariableTargetNode) {
        ruby_prism::visit_global_variable_target_node(self, node);
        analyze::global_variable_target_node(node, self);
    }
    fn visit_global_variable_write_node(&mut self, node: &ruby_prism::GlobalVariableWriteNode) {
        ruby_prism::visit_global_variable_write_node(self, node);
        analyze::global_variable_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_hash_node(&mut self, node: &ruby_prism::HashNode) {
        ruby_prism::visit_hash_node(self, node);
        analyze::hash_node(node, self);
    }
    fn visit_hash_pattern_node(&mut self, node: &ruby_prism::HashPatternNode) {
        ruby_prism::visit_hash_pattern_node(self, node);
        analyze::hash_pattern_node(node, self);
    }
    fn visit_if_node(&mut self, node: &ruby_prism::IfNode) {
        ruby_prism::visit_if_node(self, node);
        analyze::if_node(node, self);
    }
    fn visit_imaginary_node(&mut self, node: &ruby_prism::ImaginaryNode) {
        ruby_prism::visit_imaginary_node(self, node);
        analyze::imaginary_node(node, self);
    }
    fn visit_implicit_node(&mut self, node: &ruby_prism::ImplicitNode) {
        ruby_prism::visit_implicit_node(self, node);
        analyze::implicit_node(node, self);
    }
    fn visit_implicit_rest_node(&mut self, node: &ruby_prism::ImplicitRestNode) {
        ruby_prism::visit_implicit_rest_node(self, node);
        analyze::implicit_rest_node(node, self);
    }
    fn visit_in_node(&mut self, node: &ruby_prism::InNode) {
        ruby_prism::visit_in_node(self, node);
        analyze::in_node(node, self);
    }
    fn visit_index_and_write_node(&mut self, node: &ruby_prism::IndexAndWriteNode) {
        ruby_prism::visit_index_and_write_node(self, node);
        analyze::index_and_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_index_operator_write_node(&mut self, node: &ruby_prism::IndexOperatorWriteNode) {
        ruby_prism::visit_index_operator_write_node(self, node);
        analyze::index_operator_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_index_or_write_node(&mut self, node: &ruby_prism::IndexOrWriteNode) {
        ruby_prism::visit_index_or_write_node(self, node);
        analyze::index_or_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_index_target_node(&mut self, node: &ruby_prism::IndexTargetNode) {
        ruby_prism::visit_index_target_node(self, node);
        analyze::index_target_node(node, self);
    }
    fn visit_instance_variable_and_write_node(&mut self, node: &ruby_prism::InstanceVariableAndWriteNode) {
        ruby_prism::visit_instance_variable_and_write_node(self, node);
        analyze::instance_variable_and_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_instance_variable_operator_write_node(&mut self, node: &ruby_prism::InstanceVariableOperatorWriteNode) {
        ruby_prism::visit_instance_variable_operator_write_node(self, node);
        analyze::instance_variable_operator_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_instance_variable_or_write_node(&mut self, node: &ruby_prism::InstanceVariableOrWriteNode) {
        ruby_prism::visit_instance_variable_or_write_node(self, node);
        analyze::instance_variable_or_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_instance_variable_read_node(&mut self, node: &ruby_prism::InstanceVariableReadNode) {
        ruby_prism::visit_instance_variable_read_node(self, node);
        analyze::instance_variable_read_node(node, self);
    }
    fn visit_instance_variable_target_node(&mut self, node: &ruby_prism::InstanceVariableTargetNode) {
        ruby_prism::visit_instance_variable_target_node(self, node);
        analyze::instance_variable_target_node(node, self);
    }
    fn visit_instance_variable_write_node(&mut self, node: &ruby_prism::InstanceVariableWriteNode) {
        ruby_prism::visit_instance_variable_write_node(self, node);
        analyze::instance_variable_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_integer_node(&mut self, node: &ruby_prism::IntegerNode) {
        ruby_prism::visit_integer_node(self, node);
        analyze::integer_node(node, self);
    }
    fn visit_interpolated_match_last_line_node(&mut self, node: &ruby_prism::InterpolatedMatchLastLineNode) {
        ruby_prism::visit_interpolated_match_last_line_node(self, node);
        analyze::interpolated_match_last_line_node(node, self);
    }
    fn visit_interpolated_regular_expression_node(&mut self, node: &ruby_prism::InterpolatedRegularExpressionNode) {
        ruby_prism::visit_interpolated_regular_expression_node(self, node);
        analyze::interpolated_regular_expression_node(node, self);
    }
    fn visit_interpolated_string_node(&mut self, node: &ruby_prism::InterpolatedStringNode) {
        ruby_prism::visit_interpolated_string_node(self, node);
        analyze::interpolated_string_node(node, self);
    }
    fn visit_interpolated_symbol_node(&mut self, node: &ruby_prism::InterpolatedSymbolNode) {
        ruby_prism::visit_interpolated_symbol_node(self, node);
        analyze::interpolated_symbol_node(node, self);
    }
    fn visit_interpolated_x_string_node(&mut self, node: &ruby_prism::InterpolatedXStringNode) {
        ruby_prism::visit_interpolated_x_string_node(self, node);
        analyze::interpolated_x_string_node(node, self);
    }
    fn visit_it_local_variable_read_node(&mut self, node: &ruby_prism::ItLocalVariableReadNode) {
        ruby_prism::visit_it_local_variable_read_node(self, node);
        analyze::it_local_variable_read_node(node, self);
    }
    fn visit_it_parameters_node(&mut self, node: &ruby_prism::ItParametersNode) {
        ruby_prism::visit_it_parameters_node(self, node);
        analyze::it_parameters_node(node, self);
    }
    fn visit_keyword_hash_node(&mut self, node: &ruby_prism::KeywordHashNode) {
        ruby_prism::visit_keyword_hash_node(self, node);
        analyze::keyword_hash_node(node, self);
    }
    fn visit_keyword_rest_parameter_node(&mut self, node: &ruby_prism::KeywordRestParameterNode) {
        ruby_prism::visit_keyword_rest_parameter_node(self, node);
        analyze::keyword_rest_parameter_node(node, self);
    }
    fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode) {
        ruby_prism::visit_lambda_node(self, node);
        analyze::lambda_node(node, self);
    }
    fn visit_local_variable_and_write_node(&mut self, node: &ruby_prism::LocalVariableAndWriteNode) {
        ruby_prism::visit_local_variable_and_write_node(self, node);
        analyze::local_variable_and_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_local_variable_operator_write_node(&mut self, node: &ruby_prism::LocalVariableOperatorWriteNode) {
        ruby_prism::visit_local_variable_operator_write_node(self, node);
        analyze::local_variable_operator_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_local_variable_or_write_node(&mut self, node: &ruby_prism::LocalVariableOrWriteNode) {
        ruby_prism::visit_local_variable_or_write_node(self, node);
        analyze::local_variable_or_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode) {
        ruby_prism::visit_local_variable_read_node(self, node);
        analyze::local_variable_read_node(node, self);
    }
    fn visit_local_variable_target_node(&mut self, node: &ruby_prism::LocalVariableTargetNode) {
        ruby_prism::visit_local_variable_target_node(self, node);
        analyze::local_variable_target_node(node, self);
    }
    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode) {
        ruby_prism::visit_local_variable_write_node(self, node);
        analyze::local_variable_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_match_last_line_node(&mut self, node: &ruby_prism::MatchLastLineNode) {
        ruby_prism::visit_match_last_line_node(self, node);
        analyze::match_last_line_node(node, self);
    }
    fn visit_match_predicate_node(&mut self, node: &ruby_prism::MatchPredicateNode) {
        ruby_prism::visit_match_predicate_node(self, node);
        analyze::match_predicate_node(node, self);
    }
    fn visit_match_required_node(&mut self, node: &ruby_prism::MatchRequiredNode) {
        ruby_prism::visit_match_required_node(self, node);
        analyze::match_required_node(node, self);
    }
    fn visit_match_write_node(&mut self, node: &ruby_prism::MatchWriteNode) {
        ruby_prism::visit_match_write_node(self, node);
        analyze::match_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.call().as_node(), self);
    }
    fn visit_missing_node(&mut self, node: &ruby_prism::MissingNode) {
        ruby_prism::visit_missing_node(self, node);
        analyze::missing_node(node, self);
    }
    fn visit_module_node(&mut self, node: &ruby_prism::ModuleNode) {
        ruby_prism::visit_module_node(self, node);
        analyze::module_node(node, self);
    }
    fn visit_multi_target_node(&mut self, node: &ruby_prism::MultiTargetNode) {
        ruby_prism::visit_multi_target_node(self, node);
        analyze::multi_target_node(node, self);
    }
    fn visit_multi_write_node(&mut self, node: &ruby_prism::MultiWriteNode) {
        ruby_prism::visit_multi_write_node(self, node);
        analyze::multi_write_node(node, self);
        analyze::assignment(&node.as_node(), &node.value(), self);
    }
    fn visit_next_node(&mut self, node: &ruby_prism::NextNode) {
        ruby_prism::visit_next_node(self, node);
        analyze::next_node(node, self);
    }
    fn visit_nil_node(&mut self, node: &ruby_prism::NilNode) {
        ruby_prism::visit_nil_node(self, node);
        analyze::nil_node(node, self);
    }
    fn visit_no_keywords_parameter_node(&mut self, node: &ruby_prism::NoKeywordsParameterNode) {
        ruby_prism::visit_no_keywords_parameter_node(self, node);
        analyze::no_keywords_parameter_node(node, self);
    }
    fn visit_numbered_parameters_node(&mut self, node: &ruby_prism::NumberedParametersNode) {
        ruby_prism::visit_numbered_parameters_node(self, node);
        analyze::numbered_parameters_node(node, self);
    }
    fn visit_numbered_reference_read_node(&mut self, node: &ruby_prism::NumberedReferenceReadNode) {
        ruby_prism::visit_numbered_reference_read_node(self, node);
        analyze::numbered_reference_read_node(node, self);
    }
    fn visit_optional_keyword_parameter_node(&mut self, node: &ruby_prism::OptionalKeywordParameterNode) {
        ruby_prism::visit_optional_keyword_parameter_node(self, node);
        analyze::optional_keyword_parameter_node(node, self);
    }
    fn visit_optional_parameter_node(&mut self, node: &ruby_prism::OptionalParameterNode) {
        ruby_prism::visit_optional_parameter_node(self, node);
        analyze::optional_parameter_node(node, self);
    }
    fn visit_or_node(&mut self, node: &ruby_prism::OrNode) {
        ruby_prism::visit_or_node(self, node);
        analyze::or_node(node, self);
    }
    fn visit_parameters_node(&mut self, node: &ruby_prism::ParametersNode) {
        ruby_prism::visit_parameters_node(self, node);
        analyze::parameters_node(node, self);
    }
    fn visit_parentheses_node(&mut self, node: &ruby_prism::ParenthesesNode) {
        ruby_prism::visit_parentheses_node(self, node);
        analyze::parentheses_node(node, self);
    }
    fn visit_pinned_expression_node(&mut self, node: &ruby_prism::PinnedExpressionNode) {
        ruby_prism::visit_pinned_expression_node(self, node);
        analyze::pinned_expression_node(node, self);
    }
    fn visit_pinned_variable_node(&mut self, node: &ruby_prism::PinnedVariableNode) {
        ruby_prism::visit_pinned_variable_node(self, node);
        analyze::pinned_variable_node(node, self);
    }
    fn visit_post_execution_node(&mut self, node: &ruby_prism::PostExecutionNode) {
        ruby_prism::visit_post_execution_node(self, node);
        analyze::post_execution_node(node, self);
    }
    fn visit_pre_execution_node(&mut self, node: &ruby_prism::PreExecutionNode) {
        ruby_prism::visit_pre_execution_node(self, node);
        analyze::pre_execution_node(node, self);
    }
    fn visit_program_node(&mut self, node: &ruby_prism::ProgramNode) {
        ruby_prism::visit_program_node(self, node);
        analyze::program_node(node, self);
    }
    fn visit_range_node(&mut self, node: &ruby_prism::RangeNode) {
        ruby_prism::visit_range_node(self, node);
        analyze::range_node(node, self);
    }
    fn visit_rational_node(&mut self, node: &ruby_prism::RationalNode) {
        ruby_prism::visit_rational_node(self, node);
        analyze::rational_node(node, self);
    }
    fn visit_redo_node(&mut self, node: &ruby_prism::RedoNode) {
        ruby_prism::visit_redo_node(self, node);
        analyze::redo_node(node, self);
    }
    fn visit_regular_expression_node(&mut self, node: &ruby_prism::RegularExpressionNode) {
        ruby_prism::visit_regular_expression_node(self, node);
        analyze::regular_expression_node(node, self);
    }
    fn visit_required_keyword_parameter_node(&mut self, node: &ruby_prism::RequiredKeywordParameterNode) {
        ruby_prism::visit_required_keyword_parameter_node(self, node);
        analyze::required_keyword_parameter_node(node, self);
    }
    fn visit_required_parameter_node(&mut self, node: &ruby_prism::RequiredParameterNode) {
        ruby_prism::visit_required_parameter_node(self, node);
        analyze::required_parameter_node(node, self);
    }
    fn visit_rescue_modifier_node(&mut self, node: &ruby_prism::RescueModifierNode) {
        ruby_prism::visit_rescue_modifier_node(self, node);
        analyze::rescue_modifier_node(node, self);
    }
    fn visit_rescue_node(&mut self, node: &ruby_prism::RescueNode) {
        ruby_prism::visit_rescue_node(self, node);
        analyze::rescue_node(node, self);
    }
    fn visit_rest_parameter_node(&mut self, node: &ruby_prism::RestParameterNode) {
        ruby_prism::visit_rest_parameter_node(self, node);
        analyze::rest_parameter_node(node, self);
    }
    fn visit_retry_node(&mut self, node: &ruby_prism::RetryNode) {
        ruby_prism::visit_retry_node(self, node);
        analyze::retry_node(node, self);
    }
    fn visit_return_node(&mut self, node: &ruby_prism::ReturnNode) {
        ruby_prism::visit_return_node(self, node);
        analyze::return_node(node, self);
    }
    fn visit_self_node(&mut self, node: &ruby_prism::SelfNode) {
        ruby_prism::visit_self_node(self, node);
        analyze::self_node(node, self);
    }
    fn visit_shareable_constant_node(&mut self, node: &ruby_prism::ShareableConstantNode) {
        ruby_prism::visit_shareable_constant_node(self, node);
        analyze::shareable_constant_node(node, self);
    }
    fn visit_singleton_class_node(&mut self, node: &ruby_prism::SingletonClassNode) {
        ruby_prism::visit_singleton_class_node(self, node);
        analyze::singleton_class_node(node, self);
    }
    fn visit_source_encoding_node(&mut self, node: &ruby_prism::SourceEncodingNode) {
        ruby_prism::visit_source_encoding_node(self, node);
        analyze::source_encoding_node(node, self);
    }
    fn visit_source_file_node(&mut self, node: &ruby_prism::SourceFileNode) {
        ruby_prism::visit_source_file_node(self, node);
        analyze::source_file_node(node, self);
    }
    fn visit_source_line_node(&mut self, node: &ruby_prism::SourceLineNode) {
        ruby_prism::visit_source_line_node(self, node);
        analyze::source_line_node(node, self);
    }
    fn visit_splat_node(&mut self, node: &ruby_prism::SplatNode) {
        ruby_prism::visit_splat_node(self, node);
        analyze::splat_node(node, self);
    }
    fn visit_statements_node(&mut self, node: &ruby_prism::StatementsNode) {
        ruby_prism::visit_statements_node(self, node);
        analyze::statements_node(node, self);
    }
    fn visit_string_node(&mut self, node: &ruby_prism::StringNode) {
        ruby_prism::visit_string_node(self, node);
        analyze::string_node(node, self);
    }
    fn visit_super_node(&mut self, node: &ruby_prism::SuperNode) {
        ruby_prism::visit_super_node(self, node);
        analyze::super_node(node, self);
    }
    fn visit_symbol_node(&mut self, node: &ruby_prism::SymbolNode) {
        ruby_prism::visit_symbol_node(self, node);
        analyze::symbol_node(node, self);
    }
    fn visit_true_node(&mut self, node: &ruby_prism::TrueNode) {
        ruby_prism::visit_true_node(self, node);
        analyze::true_node(node, self);
    }
    fn visit_undef_node(&mut self, node: &ruby_prism::UndefNode) {
        ruby_prism::visit_undef_node(self, node);
        analyze::undef_node(node, self);
    }
    fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode) {
        ruby_prism::visit_unless_node(self, node);
        analyze::unless_node(node, self);
    }
    fn visit_until_node(&mut self, node: &ruby_prism::UntilNode) {
        ruby_prism::visit_until_node(self, node);
        analyze::until_node(node, self);
    }
    fn visit_when_node(&mut self, node: &ruby_prism::WhenNode) {
        ruby_prism::visit_when_node(self, node);
        analyze::when_node(node, self);
    }
    fn visit_while_node(&mut self, node: &ruby_prism::WhileNode) {
        ruby_prism::visit_while_node(self, node);
        analyze::while_node(node, self);
    }
    fn visit_x_string_node(&mut self, node: &ruby_prism::XStringNode) {
        ruby_prism::visit_x_string_node(self, node);
        analyze::x_string_node(node, self);
    }
    fn visit_yield_node(&mut self, node: &ruby_prism::YieldNode) {
        ruby_prism::visit_yield_node(self, node);
        analyze::yield_node(node, self);
    }
}
