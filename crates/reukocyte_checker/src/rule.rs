use std::fmt;

use crate::checker::Checker;
use ruby_prism::*;

/// Unique identifier for a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RuleId {
    Layout(LayoutRule),
    Lint(LintRule),
}
impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.category().as_str(), self.name())
    }
}

impl RuleId {
    /// Get the category of the rule.
    pub const fn category(&self) -> Category {
        match self {
            Self::Layout(_) => Category::Layout,
            Self::Lint(_) => Category::Lint,
        }
    }
    /// Get the rule name without category.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Layout(rule) => rule.name(),
            Self::Lint(rule) => rule.name(),
        }
    }
    /// Rules that this rule's autocorrection conflicts with.
    ///
    /// When two rules conflict, only one of them should have its
    /// fixes applied in a single iteration. The skipped rule's fixes will
    /// be applied in a subsequent iteration.
    ///
    /// This is equivalent to RuboCop's `autocorrect_incompatible_with`.
    pub const fn conflicts_with(&self) -> &'static [RuleId] {
        match self {
            Self::Layout(LayoutRule::EmptyLines) => &[],
            Self::Layout(LayoutRule::EndAlignment) => &[],
            Self::Layout(LayoutRule::IndentationStyle) => &[],
            Self::Layout(LayoutRule::IndentationWidth) => &[],
            Self::Layout(LayoutRule::LeadingEmptyLines) => &[],
            Self::Layout(LayoutRule::TrailingEmptyLines) => &[],
            Self::Layout(LayoutRule::TrailingWhitespace) => &[],
            Self::Lint(LintRule::Debugger) => &[],
        }
    }
    /// Check if this rule conflicts with another rule.
    pub fn has_conflict_with(&self, other: RuleId) -> bool {
        self.conflicts_with().contains(&other)
    }
}

/// Category of a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Category {
    Layout,
    Lint,
}
impl Category {
    /// Get the category name as a string.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Layout => "Layout",
            Self::Lint => "Lint",
        }
    }
}

/// Layout rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LayoutRule {
    EmptyLines,
    EndAlignment,
    IndentationStyle,
    IndentationWidth,
    LeadingEmptyLines,
    TrailingEmptyLines,
    TrailingWhitespace,
}
impl LayoutRule {
    /// Get the rule name.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::EmptyLines => "EmptyLines",
            Self::EndAlignment => "EndAlignment",
            Self::IndentationStyle => "IndentationStyle",
            Self::IndentationWidth => "IndentationWidth",
            Self::LeadingEmptyLines => "LeadingEmptyLines",
            Self::TrailingEmptyLines => "TrailingEmptyLines",
            Self::TrailingWhitespace => "TrailingWhitespace",
        }
    }
}

/// Lint rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LintRule {
    Debugger,
}
impl LintRule {
    /// Get the rule name.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Debugger => "Debugger",
        }
    }
}

// ============================================================================
// Rule Traits
// ============================================================================

/// Base trait for all rules.
///
/// Each rule struct should implement this trait to provide its identifier
/// and optionally declare conflicting rules.
pub trait Rule {
    const ID: RuleId;
    const CONFLICTS: &'static [RuleId] = &[];
}
/// Trait for rules that check [`AliasGlobalVariableNode`].
pub trait CheckAliasGlobalVariableNode: Rule {
    fn check(node: &AliasGlobalVariableNode, checker: &mut Checker);
}
/// Trait for rules that check [`AliasMethodNode`].
pub trait CheckAliasMethodNode: Rule {
    fn check(node: &AliasMethodNode, checker: &mut Checker);
}
/// Trait for rules that check [`AlternationPatternNode`].
pub trait CheckAlternationPatternNode: Rule {
    fn check(node: &AlternationPatternNode, checker: &mut Checker);
}
/// Trait for rules that check [`AndNode`].
pub trait CheckAndNode: Rule {
    fn check(node: &AndNode, checker: &mut Checker);
}
/// Trait for rules that check [`ArgumentsNode`].
pub trait CheckArgumentsNode: Rule {
    fn check(node: &ArgumentsNode, checker: &mut Checker);
}
/// Trait for rules that check [`ArrayNode`].
pub trait CheckArrayNode: Rule {
    fn check(node: &ArrayNode, checker: &mut Checker);
}
/// Trait for rules that check [`ArrayPatternNode`].
pub trait CheckArrayPatternNode: Rule {
    fn check(node: &ArrayPatternNode, checker: &mut Checker);
}
/// Trait for rules that check [`AssocNode`].
pub trait CheckAssocNode: Rule {
    fn check(node: &AssocNode, checker: &mut Checker);
}
/// Trait for rules that check [`AssocSplatNode`].
pub trait CheckAssocSplatNode: Rule {
    fn check(node: &AssocSplatNode, checker: &mut Checker);
}
/// Trait for rules that check [`BackReferenceReadNode`].
pub trait CheckBackReferenceReadNode: Rule {
    fn check(node: &BackReferenceReadNode, checker: &mut Checker);
}
/// Trait for rules that check [`BeginNode`].
pub trait CheckBeginNode: Rule {
    fn check(node: &BeginNode, checker: &mut Checker);
}
/// Trait for rules that check [`BlockArgumentNode`].
pub trait CheckBlockArgumentNode: Rule {
    fn check(node: &BlockArgumentNode, checker: &mut Checker);
}
/// Trait for rules that check [`BlockLocalVariableNode`].
pub trait CheckBlockLocalVariableNode: Rule {
    fn check(node: &BlockLocalVariableNode, checker: &mut Checker);
}
/// Trait for rules that check [`BlockNode`].
pub trait CheckBlockNode: Rule {
    fn check(node: &BlockNode, checker: &mut Checker);
}
/// Trait for rules that check [`BlockParameterNode`].
pub trait CheckBlockParameterNode: Rule {
    fn check(node: &BlockParameterNode, checker: &mut Checker);
}
/// Trait for rules that check [`BlockParametersNode`].
pub trait CheckBlockParametersNode: Rule {
    fn check(node: &BlockParametersNode, checker: &mut Checker);
}
/// Trait for rules that check [`BreakNode`].
pub trait CheckBreakNode: Rule {
    fn check(node: &BreakNode, checker: &mut Checker);
}
/// Trait for rules that check [`CallAndWriteNode`].
pub trait CheckCallAndWriteNode: Rule {
    fn check(node: &CallAndWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`CallNode`].
pub trait CheckCallNode: Rule {
    fn check(node: &CallNode, checker: &mut Checker);
}
/// Trait for rules that check [`CallOperatorWriteNode`].
pub trait CheckCallOperatorWriteNode: Rule {
    fn check(node: &CallOperatorWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`CallOrWriteNode`].
pub trait CheckCallOrWriteNode: Rule {
    fn check(node: &CallOrWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`CallTargetNode`].
pub trait CheckCallTargetNode: Rule {
    fn check(node: &CallTargetNode, checker: &mut Checker);
}
/// Trait for rules that check [`CapturePatternNode`].
pub trait CheckCapturePatternNode: Rule {
    fn check(node: &CapturePatternNode, checker: &mut Checker);
}
/// Trait for rules that check [`CaseMatchNode`].
pub trait CheckCaseMatchNode: Rule {
    fn check(node: &CaseMatchNode, checker: &mut Checker);
}
/// Trait for rules that check [`CaseNode`].
pub trait CheckCaseNode: Rule {
    fn check(node: &CaseNode, checker: &mut Checker);
}
/// Trait for rules that check [`ClassNode`].
pub trait CheckClassNode: Rule {
    fn check(node: &ClassNode, checker: &mut Checker);
}
/// Trait for rules that check [`ClassVariableAndWriteNode`].
pub trait CheckClassVariableAndWriteNode: Rule {
    fn check(node: &ClassVariableAndWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`ClassVariableOperatorWriteNode`].
pub trait CheckClassVariableOperatorWriteNode: Rule {
    fn check(node: &ClassVariableOperatorWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`ClassVariableOrWriteNode`].
pub trait CheckClassVariableOrWriteNode: Rule {
    fn check(node: &ClassVariableOrWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`ClassVariableReadNode`].
pub trait CheckClassVariableReadNode: Rule {
    fn check(node: &ClassVariableReadNode, checker: &mut Checker);
}
/// Trait for rules that check [`ClassVariableTargetNode`].
pub trait CheckClassVariableTargetNode: Rule {
    fn check(node: &ClassVariableTargetNode, checker: &mut Checker);
}
/// Trait for rules that check [`ClassVariableWriteNode`].
pub trait CheckClassVariableWriteNode: Rule {
    fn check(node: &ClassVariableWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`ConstantAndWriteNode`].
pub trait CheckConstantAndWriteNode: Rule {
    fn check(node: &ConstantAndWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`ConstantOperatorWriteNode`].
pub trait CheckConstantOperatorWriteNode: Rule {
    fn check(node: &ConstantOperatorWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`ConstantOrWriteNode`].
pub trait CheckConstantOrWriteNode: Rule {
    fn check(node: &ConstantOrWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`ConstantPathAndWriteNode`].
pub trait CheckConstantPathAndWriteNode: Rule {
    fn check(node: &ConstantPathAndWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`ConstantPathNode`].
pub trait CheckConstantPathNode: Rule {
    fn check(node: &ConstantPathNode, checker: &mut Checker);
}
/// Trait for rules that check [`ConstantPathOperatorWriteNode`].
pub trait CheckConstantPathOperatorWriteNode: Rule {
    fn check(node: &ConstantPathOperatorWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`ConstantPathOrWriteNode`].
pub trait CheckConstantPathOrWriteNode: Rule {
    fn check(node: &ConstantPathOrWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`ConstantPathTargetNode`].
pub trait CheckConstantPathTargetNode: Rule {
    fn check(node: &ConstantPathTargetNode, checker: &mut Checker);
}
/// Trait for rules that check [`ConstantPathWriteNode`].
pub trait CheckConstantPathWriteNode: Rule {
    fn check(node: &ConstantPathWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`ConstantReadNode`].
pub trait CheckConstantReadNode: Rule {
    fn check(node: &ConstantReadNode, checker: &mut Checker);
}
/// Trait for rules that check [`ConstantTargetNode`].
pub trait CheckConstantTargetNode: Rule {
    fn check(node: &ConstantTargetNode, checker: &mut Checker);
}
/// Trait for rules that check [`ConstantWriteNode`].
pub trait CheckConstantWriteNode: Rule {
    fn check(node: &ConstantWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`DefNode`].
pub trait CheckDefNode: Rule {
    fn check(node: &DefNode, checker: &mut Checker);
}
/// Trait for rules that check [`DefinedNode`].
pub trait CheckDefinedNode: Rule {
    fn check(node: &DefinedNode, checker: &mut Checker);
}
/// Trait for rules that check [`ElseNode`].
pub trait CheckElseNode: Rule {
    fn check(node: &ElseNode, checker: &mut Checker);
}
/// Trait for rules that check [`EmbeddedStatementsNode`].
pub trait CheckEmbeddedStatementsNode: Rule {
    fn check(node: &EmbeddedStatementsNode, checker: &mut Checker);
}
/// Trait for rules that check [`EmbeddedVariableNode`].
pub trait CheckEmbeddedVariableNode: Rule {
    fn check(node: &EmbeddedVariableNode, checker: &mut Checker);
}
/// Trait for rules that check [`EnsureNode`].
pub trait CheckEnsureNode: Rule {
    fn check(node: &EnsureNode, checker: &mut Checker);
}
/// Trait for rules that check [`FalseNode`].
pub trait CheckFalseNode: Rule {
    fn check(node: &FalseNode, checker: &mut Checker);
}
/// Trait for rules that check [`FindPatternNode`].
pub trait CheckFindPatternNode: Rule {
    fn check(node: &FindPatternNode, checker: &mut Checker);
}
/// Trait for rules that check [`FlipFlopNode`].
pub trait CheckFlipFlopNode: Rule {
    fn check(node: &FlipFlopNode, checker: &mut Checker);
}
/// Trait for rules that check [`FloatNode`].
pub trait CheckFloatNode: Rule {
    fn check(node: &FloatNode, checker: &mut Checker);
}
/// Trait for rules that check [`ForNode`].
pub trait CheckForNode: Rule {
    fn check(node: &ForNode, checker: &mut Checker);
}
/// Trait for rules that check [`ForwardingArgumentsNode`].
pub trait CheckForwardingArgumentsNode: Rule {
    fn check(node: &ForwardingArgumentsNode, checker: &mut Checker);
}
/// Trait for rules that check [`ForwardingParameterNode`].
pub trait CheckForwardingParameterNode: Rule {
    fn check(node: &ForwardingParameterNode, checker: &mut Checker);
}
/// Trait for rules that check [`ForwardingSuperNode`].
pub trait CheckForwardingSuperNode: Rule {
    fn check(node: &ForwardingSuperNode, checker: &mut Checker);
}
/// Trait for rules that check [`GlobalVariableAndWriteNode`].
pub trait CheckGlobalVariableAndWriteNode: Rule {
    fn check(node: &GlobalVariableAndWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`GlobalVariableOperatorWriteNode`].
pub trait CheckGlobalVariableOperatorWriteNode: Rule {
    fn check(node: &GlobalVariableOperatorWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`GlobalVariableOrWriteNode`].
pub trait CheckGlobalVariableOrWriteNode: Rule {
    fn check(node: &GlobalVariableOrWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`GlobalVariableReadNode`].
pub trait CheckGlobalVariableReadNode: Rule {
    fn check(node: &GlobalVariableReadNode, checker: &mut Checker);
}
/// Trait for rules that check [`GlobalVariableTargetNode`].
pub trait CheckGlobalVariableTargetNode: Rule {
    fn check(node: &GlobalVariableTargetNode, checker: &mut Checker);
}
/// Trait for rules that check [`GlobalVariableWriteNode`].
pub trait CheckGlobalVariableWriteNode: Rule {
    fn check(node: &GlobalVariableWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`HashNode`].
pub trait CheckHashNode: Rule {
    fn check(node: &HashNode, checker: &mut Checker);
}
/// Trait for rules that check [`HashPatternNode`].
pub trait CheckHashPatternNode: Rule {
    fn check(node: &HashPatternNode, checker: &mut Checker);
}
/// Trait for rules that check [`IfNode`].
pub trait CheckIfNode: Rule {
    fn check(node: &IfNode, checker: &mut Checker);
}
/// Trait for rules that check [`ImaginaryNode`].
pub trait CheckImaginaryNode: Rule {
    fn check(node: &ImaginaryNode, checker: &mut Checker);
}
/// Trait for rules that check [`ImplicitNode`].
pub trait CheckImplicitNode: Rule {
    fn check(node: &ImplicitNode, checker: &mut Checker);
}
/// Trait for rules that check [`ImplicitRestNode`].
pub trait CheckImplicitRestNode: Rule {
    fn check(node: &ImplicitRestNode, checker: &mut Checker);
}
/// Trait for rules that check [`InNode`].
pub trait CheckInNode: Rule {
    fn check(node: &InNode, checker: &mut Checker);
}
/// Trait for rules that check [`IndexAndWriteNode`].
pub trait CheckIndexAndWriteNode: Rule {
    fn check(node: &IndexAndWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`IndexOperatorWriteNode`].
pub trait CheckIndexOperatorWriteNode: Rule {
    fn check(node: &IndexOperatorWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`IndexOrWriteNode`].
pub trait CheckIndexOrWriteNode: Rule {
    fn check(node: &IndexOrWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`IndexTargetNode`].
pub trait CheckIndexTargetNode: Rule {
    fn check(node: &IndexTargetNode, checker: &mut Checker);
}
/// Trait for rules that check [`InstanceVariableAndWriteNode`].
pub trait CheckInstanceVariableAndWriteNode: Rule {
    fn check(node: &InstanceVariableAndWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`InstanceVariableOperatorWriteNode`].
pub trait CheckInstanceVariableOperatorWriteNode: Rule {
    fn check(node: &InstanceVariableOperatorWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`InstanceVariableOrWriteNode`].
pub trait CheckInstanceVariableOrWriteNode: Rule {
    fn check(node: &InstanceVariableOrWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`InstanceVariableReadNode`].
pub trait CheckInstanceVariableReadNode: Rule {
    fn check(node: &InstanceVariableReadNode, checker: &mut Checker);
}
/// Trait for rules that check [`InstanceVariableTargetNode`].
pub trait CheckInstanceVariableTargetNode: Rule {
    fn check(node: &InstanceVariableTargetNode, checker: &mut Checker);
}
/// Trait for rules that check [`InstanceVariableWriteNode`].
pub trait CheckInstanceVariableWriteNode: Rule {
    fn check(node: &InstanceVariableWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`IntegerNode`].
pub trait CheckIntegerNode: Rule {
    fn check(node: &IntegerNode, checker: &mut Checker);
}
/// Trait for rules that check [`InterpolatedMatchLastLineNode`].
pub trait CheckInterpolatedMatchLastLineNode: Rule {
    fn check(node: &InterpolatedMatchLastLineNode, checker: &mut Checker);
}
/// Trait for rules that check [`InterpolatedRegularExpressionNode`].
pub trait CheckInterpolatedRegularExpressionNode: Rule {
    fn check(node: &InterpolatedRegularExpressionNode, checker: &mut Checker);
}
/// Trait for rules that check [`InterpolatedStringNode`].
pub trait CheckInterpolatedStringNode: Rule {
    fn check(node: &InterpolatedStringNode, checker: &mut Checker);
}
/// Trait for rules that check [`InterpolatedSymbolNode`].
pub trait CheckInterpolatedSymbolNode: Rule {
    fn check(node: &InterpolatedSymbolNode, checker: &mut Checker);
}
/// Trait for rules that check [`InterpolatedXStringNode`].
pub trait CheckInterpolatedXStringNode: Rule {
    fn check(node: &InterpolatedXStringNode, checker: &mut Checker);
}
/// Trait for rules that check [`ItLocalVariableReadNode`].
pub trait CheckItLocalVariableReadNode: Rule {
    fn check(node: &ItLocalVariableReadNode, checker: &mut Checker);
}
/// Trait for rules that check [`ItParametersNode`].
pub trait CheckItParametersNode: Rule {
    fn check(node: &ItParametersNode, checker: &mut Checker);
}
/// Trait for rules that check [`KeywordHashNode`].
pub trait CheckKeywordHashNode: Rule {
    fn check(node: &KeywordHashNode, checker: &mut Checker);
}
/// Trait for rules that check [`KeywordRestParameterNode`].
pub trait CheckKeywordRestParameterNode: Rule {
    fn check(node: &KeywordRestParameterNode, checker: &mut Checker);
}
/// Trait for rules that check [`LambdaNode`].
pub trait CheckLambdaNode: Rule {
    fn check(node: &LambdaNode, checker: &mut Checker);
}
/// Trait for rules that check [`LocalVariableAndWriteNode`].
pub trait CheckLocalVariableAndWriteNode: Rule {
    fn check(node: &LocalVariableAndWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`LocalVariableOperatorWriteNode`].
pub trait CheckLocalVariableOperatorWriteNode: Rule {
    fn check(node: &LocalVariableOperatorWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`LocalVariableOrWriteNode`].
pub trait CheckLocalVariableOrWriteNode: Rule {
    fn check(node: &LocalVariableOrWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`LocalVariableReadNode`].
pub trait CheckLocalVariableReadNode: Rule {
    fn check(node: &LocalVariableReadNode, checker: &mut Checker);
}
/// Trait for rules that check [`LocalVariableTargetNode`].
pub trait CheckLocalVariableTargetNode: Rule {
    fn check(node: &LocalVariableTargetNode, checker: &mut Checker);
}
/// Trait for rules that check [`LocalVariableWriteNode`].
pub trait CheckLocalVariableWriteNode: Rule {
    fn check(node: &LocalVariableWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`MatchLastLineNode`].
pub trait CheckMatchLastLineNode: Rule {
    fn check(node: &MatchLastLineNode, checker: &mut Checker);
}
/// Trait for rules that check [`MatchPredicateNode`].
pub trait CheckMatchPredicateNode: Rule {
    fn check(node: &MatchPredicateNode, checker: &mut Checker);
}
/// Trait for rules that check [`MatchRequiredNode`].
pub trait CheckMatchRequiredNode: Rule {
    fn check(node: &MatchRequiredNode, checker: &mut Checker);
}
/// Trait for rules that check [`MatchWriteNode`].
pub trait CheckMatchWriteNode: Rule {
    fn check(node: &MatchWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`MissingNode`].
pub trait CheckMissingNode: Rule {
    fn check(node: &MissingNode, checker: &mut Checker);
}
/// Trait for rules that check [`ModuleNode`].
pub trait CheckModuleNode: Rule {
    fn check(node: &ModuleNode, checker: &mut Checker);
}
/// Trait for rules that check [`MultiTargetNode`].
pub trait CheckMultiTargetNode: Rule {
    fn check(node: &MultiTargetNode, checker: &mut Checker);
}
/// Trait for rules that check [`MultiWriteNode`].
pub trait CheckMultiWriteNode: Rule {
    fn check(node: &MultiWriteNode, checker: &mut Checker);
}
/// Trait for rules that check [`NextNode`].
pub trait CheckNextNode: Rule {
    fn check(node: &NextNode, checker: &mut Checker);
}
/// Trait for rules that check [`NilNode`].
pub trait CheckNilNode: Rule {
    fn check(node: &NilNode, checker: &mut Checker);
}
/// Trait for rules that check [`NoKeywordsParameterNode`].
pub trait CheckNoKeywordsParameterNode: Rule {
    fn check(node: &NoKeywordsParameterNode, checker: &mut Checker);
}
/// Trait for rules that check [`NumberedParametersNode`].
pub trait CheckNumberedParametersNode: Rule {
    fn check(node: &NumberedParametersNode, checker: &mut Checker);
}
/// Trait for rules that check [`NumberedReferenceReadNode`].
pub trait CheckNumberedReferenceReadNode: Rule {
    fn check(node: &NumberedReferenceReadNode, checker: &mut Checker);
}
/// Trait for rules that check [`OptionalKeywordParameterNode`].
pub trait CheckOptionalKeywordParameterNode: Rule {
    fn check(node: &OptionalKeywordParameterNode, checker: &mut Checker);
}
/// Trait for rules that check [`OptionalParameterNode`].
pub trait CheckOptionalParameterNode: Rule {
    fn check(node: &OptionalParameterNode, checker: &mut Checker);
}
/// Trait for rules that check [`OrNode`].
pub trait CheckOrNode: Rule {
    fn check(node: &OrNode, checker: &mut Checker);
}
/// Trait for rules that check [`ParametersNode`].
pub trait CheckParametersNode: Rule {
    fn check(node: &ParametersNode, checker: &mut Checker);
}
/// Trait for rules that check [`ParenthesesNode`].
pub trait CheckParenthesesNode: Rule {
    fn check(node: &ParenthesesNode, checker: &mut Checker);
}
/// Trait for rules that check [`PinnedExpressionNode`].
pub trait CheckPinnedExpressionNode: Rule {
    fn check(node: &PinnedExpressionNode, checker: &mut Checker);
}
/// Trait for rules that check [`PinnedVariableNode`].
pub trait CheckPinnedVariableNode: Rule {
    fn check(node: &PinnedVariableNode, checker: &mut Checker);
}
/// Trait for rules that check [`PostExecutionNode`].
pub trait CheckPostExecutionNode: Rule {
    fn check(node: &PostExecutionNode, checker: &mut Checker);
}
/// Trait for rules that check [`PreExecutionNode`].
pub trait CheckPreExecutionNode: Rule {
    fn check(node: &PreExecutionNode, checker: &mut Checker);
}
/// Trait for rules that check [`ProgramNode`].
pub trait CheckProgramNode: Rule {
    fn check(node: &ProgramNode, checker: &mut Checker);
}
/// Trait for rules that check [`RangeNode`].
pub trait CheckRangeNode: Rule {
    fn check(node: &RangeNode, checker: &mut Checker);
}
/// Trait for rules that check [`RationalNode`].
pub trait CheckRationalNode: Rule {
    fn check(node: &RationalNode, checker: &mut Checker);
}
/// Trait for rules that check [`RedoNode`].
pub trait CheckRedoNode: Rule {
    fn check(node: &RedoNode, checker: &mut Checker);
}
/// Trait for rules that check [`RegularExpressionNode`].
pub trait CheckRegularExpressionNode: Rule {
    fn check(node: &RegularExpressionNode, checker: &mut Checker);
}
/// Trait for rules that check [`RequiredKeywordParameterNode`].
pub trait CheckRequiredKeywordParameterNode: Rule {
    fn check(node: &RequiredKeywordParameterNode, checker: &mut Checker);
}
/// Trait for rules that check [`RequiredParameterNode`].
pub trait CheckRequiredParameterNode: Rule {
    fn check(node: &RequiredParameterNode, checker: &mut Checker);
}
/// Trait for rules that check [`RescueModifierNode`].
pub trait CheckRescueModifierNode: Rule {
    fn check(node: &RescueModifierNode, checker: &mut Checker);
}
/// Trait for rules that check [`RescueNode`].
pub trait CheckRescueNode: Rule {
    fn check(node: &RescueNode, checker: &mut Checker);
}
/// Trait for rules that check [`RestParameterNode`].
pub trait CheckRestParameterNode: Rule {
    fn check(node: &RestParameterNode, checker: &mut Checker);
}
/// Trait for rules that check [`RetryNode`].
pub trait CheckRetryNode: Rule {
    fn check(node: &RetryNode, checker: &mut Checker);
}
/// Trait for rules that check [`ReturnNode`].
pub trait CheckReturnNode: Rule {
    fn check(node: &ReturnNode, checker: &mut Checker);
}
/// Trait for rules that check [`SelfNode`].
pub trait CheckSelfNode: Rule {
    fn check(node: &SelfNode, checker: &mut Checker);
}
/// Trait for rules that check [`ShareableConstantNode`].
pub trait CheckShareableConstantNode: Rule {
    fn check(node: &ShareableConstantNode, checker: &mut Checker);
}
/// Trait for rules that check [`SingletonClassNode`].
pub trait CheckSingletonClassNode: Rule {
    fn check(node: &SingletonClassNode, checker: &mut Checker);
}
/// Trait for rules that check [`SourceEncodingNode`].
pub trait CheckSourceEncodingNode: Rule {
    fn check(node: &SourceEncodingNode, checker: &mut Checker);
}
/// Trait for rules that check [`SourceFileNode`].
pub trait CheckSourceFileNode: Rule {
    fn check(node: &SourceFileNode, checker: &mut Checker);
}
/// Trait for rules that check [`SourceLineNode`].
pub trait CheckSourceLineNode: Rule {
    fn check(node: &SourceLineNode, checker: &mut Checker);
}
/// Trait for rules that check [`SplatNode`].
pub trait CheckSplatNode: Rule {
    fn check(node: &SplatNode, checker: &mut Checker);
}
/// Trait for rules that check [`StatementsNode`].
pub trait CheckStatementsNode: Rule {
    fn check(node: &StatementsNode, checker: &mut Checker);
}
/// Trait for rules that check [`StringNode`].
pub trait CheckStringNode: Rule {
    fn check(node: &StringNode, checker: &mut Checker);
}
/// Trait for rules that check [`SuperNode`].
pub trait CheckSuperNode: Rule {
    fn check(node: &SuperNode, checker: &mut Checker);
}
/// Trait for rules that check [`SymbolNode`].
pub trait CheckSymbolNode: Rule {
    fn check(node: &SymbolNode, checker: &mut Checker);
}
/// Trait for rules that check [`TrueNode`].
pub trait CheckTrueNode: Rule {
    fn check(node: &TrueNode, checker: &mut Checker);
}
/// Trait for rules that check [`UndefNode`].
pub trait CheckUndefNode: Rule {
    fn check(node: &UndefNode, checker: &mut Checker);
}
/// Trait for rules that check [`UnlessNode`].
pub trait CheckUnlessNode: Rule {
    fn check(node: &UnlessNode, checker: &mut Checker);
}
/// Trait for rules that check [`UntilNode`].
pub trait CheckUntilNode: Rule {
    fn check(node: &UntilNode, checker: &mut Checker);
}
/// Trait for rules that check [`WhenNode`].
pub trait CheckWhenNode: Rule {
    fn check(node: &WhenNode, checker: &mut Checker);
}
/// Trait for rules that check [`WhileNode`].
pub trait CheckWhileNode: Rule {
    fn check(node: &WhileNode, checker: &mut Checker);
}
/// Trait for rules that check [`XStringNode`].
pub trait CheckXStringNode: Rule {
    fn check(node: &XStringNode, checker: &mut Checker);
}
/// Trait for rules that check [`YieldNode`].
pub trait CheckYieldNode: Rule {
    fn check(node: &YieldNode, checker: &mut Checker);
}

// ============================================================================
// Rule Runner Macro
// ============================================================================

/// Macro to run multiple rules for a specific node type.
///
/// This macro generates static dispatch calls for each rule, avoiding
/// the overhead of dynamic dispatch (trait objects).
///
/// # Example
/// ```ignore
/// run_rules!(node, checker, CheckClassNode, [
///     EndAlignment,
///     IndentationWidth,
/// ]);
/// ```
#[macro_export]
macro_rules! run_rules {
    ($node:expr, $checker:expr, $trait:ident, [$($rule:ty),* $(,)?]) => {
        $(
            if $checker.is_enabled(<$rule as $crate::rule::Rule>::ID) {
                <$rule as $crate::rule::$trait>::check($node, $checker);
            }
        )*
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_id_parts() {
        let layout_rule = RuleId::Layout(LayoutRule::TrailingWhitespace);
        let lint_rule = RuleId::Lint(LintRule::Debugger);

        assert_eq!(layout_rule.category(), Category::Layout);
        assert_eq!(layout_rule.name(), "TrailingWhitespace");
        assert_eq!(lint_rule.category(), Category::Lint);
        assert_eq!(lint_rule.name(), "Debugger");
    }

    #[test]
    fn test_rule_id_display() {
        assert_eq!(
            format!(
                "{}/{}",
                RuleId::Layout(LayoutRule::TrailingWhitespace).category().as_str(),
                RuleId::Layout(LayoutRule::TrailingWhitespace).name()
            ),
            "Layout/TrailingWhitespace"
        );
        assert_eq!(
            format!(
                "{}/{}",
                RuleId::Lint(LintRule::Debugger).category().as_str(),
                RuleId::Lint(LintRule::Debugger).name()
            ),
            "Lint/Debugger"
        );
    }

    #[test]
    fn test_no_conflict() {
        let rule = RuleId::Layout(LayoutRule::TrailingWhitespace);
        assert!(!rule.has_conflict_with(RuleId::Lint(LintRule::Debugger)));
    }

    #[test]
    fn test_rule_id_equality() {
        assert_eq!(RuleId::Layout(LayoutRule::TrailingWhitespace), RuleId::Layout(LayoutRule::TrailingWhitespace));
        assert_ne!(RuleId::Layout(LayoutRule::TrailingWhitespace), RuleId::Lint(LintRule::Debugger));
    }
}
