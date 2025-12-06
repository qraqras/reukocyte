# Reukocyte

## お願い
- 特に指定がない限り日本語で回答してください
- できるだけ図示してください
- コードの修正をする場合、事前に方針を説明してください

## 直近のタスク
このプロジェクトの直近のタスクは以下の通りです:
- EndAlignmentの実装を見直し
- Layoutの実装
  - [ ] Layout/AccessModifierIndentation
  - [ ] Layout/ArgumentAlignment
  - [ ] Layout/ArrayAlignment
  - [ ] Layout/AssignmentIndentation
  - [ ] Layout/BeginEndAlignment
  - [ ] Layout/BlockAlignment
  - [ ] Layout/BlockEndNewline
  - [ ] Layout/CaseIndentation
  - [ ] Layout/ClassStructure
  - [ ] Layout/ClosingHeredocIndentation
  - [ ] Layout/ClosingParenthesisIndentation
  - [ ] Layout/CommentIndentation
  - [ ] Layout/ConditionPosition
  - [ ] Layout/DefEndAlignment
  - [ ] Layout/DotPosition
  - [ ] Layout/ElseAlignment
  - [ ] Layout/EmptyComment
  - [ ] Layout/EmptyLineAfterGuardClause
  - [ ] Layout/EmptyLineAfterMagicComment
  - [ ] Layout/EmptyLineAfterMultilineCondition
  - [ ] Layout/EmptyLineBetweenDefs
  - [ ] Layout/EmptyLines
  - [ ] Layout/EmptyLinesAfterModuleInclusion
  - [ ] Layout/EmptyLinesAroundAccessModifier
  - [ ] Layout/EmptyLinesAroundArguments
  - [ ] Layout/EmptyLinesAroundAttributeAccessor
  - [ ] Layout/EmptyLinesAroundBeginBody
  - [ ] Layout/EmptyLinesAroundBlockBody
  - [ ] Layout/EmptyLinesAroundClassBody
  - [ ] Layout/EmptyLinesAroundExceptionHandlingKeywords
  - [ ] Layout/EmptyLinesAroundMethodBody
  - [ ] Layout/EmptyLinesAroundModuleBody
  - [ ] Layout/EndAlignment
  - [ ] Layout/EndOfLine
  - [ ] Layout/ExtraSpacing
  - [ ] Layout/FirstArgumentIndentation
  - [ ] Layout/FirstArrayElementIndentation
  - [ ] Layout/FirstArrayElementLineBreak
  - [ ] Layout/FirstHashElementIndentation
  - [ ] Layout/FirstHashElementLineBreak
  - [ ] Layout/FirstMethodArgumentLineBreak
  - [ ] Layout/FirstMethodParameterLineBreak
  - [ ] Layout/FirstParameterIndentation
  - [ ] Layout/HashAlignment
  - [ ] Layout/HeredocArgumentClosingParenthesis
  - [ ] Layout/HeredocIndentation
  - [ ] Layout/IndentationConsistency
  - [ ] Layout/IndentationStyle
  - [ x ] Layout/IndentationWidth
  - [ ] Layout/InitialIndentation
  - [ ] Layout/LeadingCommentSpace
  - [ ] Layout/LeadingEmptyLines
  - [ ] Layout/LineContinuationLeadingSpace
  - [ ] Layout/LineContinuationSpacing
  - [ ] Layout/LineEndStringConcatenationIndentation
  - [ ] Layout/LineLength
  - [ ] Layout/MultilineArrayBraceLayout
  - [ ] Layout/MultilineArrayLineBreaks
  - [ ] Layout/MultilineAssignmentLayout
  - [ ] Layout/MultilineBlockLayout
  - [ ] Layout/MultilineHashBraceLayout
  - [ ] Layout/MultilineHashKeyLineBreaks
  - [ ] Layout/MultilineMethodArgumentLineBreaks
  - [ ] Layout/MultilineMethodCallBraceLayout
  - [ ] Layout/MultilineMethodCallIndentation
  - [ ] Layout/MultilineMethodDefinitionBraceLayout
  - [ ] Layout/MultilineMethodParameterLineBreaks
  - [ ] Layout/MultilineOperationIndentation
  - [ ] Layout/ParameterAlignment
  - [ ] Layout/RedundantLineBreak
  - [ ] Layout/RescueEnsureAlignment
  - [ ] Layout/SingleLineBlockChain
  - [ ] Layout/SpaceAfterColon
  - [ ] Layout/SpaceAfterComma
  - [ ] Layout/SpaceAfterMethodName
  - [ ] Layout/SpaceAfterNot
  - [ ] Layout/SpaceAfterSemicolon
  - [ ] Layout/SpaceAroundBlockParameters
  - [ ] Layout/SpaceAroundEqualsInParameterDefault
  - [ ] Layout/SpaceAroundKeyword
  - [ ] Layout/SpaceAroundMethodCallOperator
  - [ ] Layout/SpaceAroundOperators
  - [ ] Layout/SpaceBeforeBlockBraces
  - [ ] Layout/SpaceBeforeBrackets
  - [ ] Layout/SpaceBeforeComma
  - [ ] Layout/SpaceBeforeComment
  - [ ] Layout/SpaceBeforeFirstArg
  - [ ] Layout/SpaceBeforeSemicolon
  - [ ] Layout/SpaceInLambdaLiteral
  - [ ] Layout/SpaceInsideArrayLiteralBrackets
  - [ ] Layout/SpaceInsideArrayPercentLiteral
  - [ ] Layout/SpaceInsideBlockBraces
  - [ ] Layout/SpaceInsideHashLiteralBraces
  - [ ] Layout/SpaceInsideParens
  - [ ] Layout/SpaceInsidePercentLiteralDelimiters
  - [ ] Layout/SpaceInsideRangeLiteral
  - [ ] Layout/SpaceInsideReferenceBrackets
  - [ ] Layout/SpaceInsideStringInterpolation
  - [ ] Layout/TrailingEmptyLines
  - [ x ] Layout/TrailingWhitespace
  - [ ] AllowMultilineFinalElement

## プロジェクトの全体像
### プロジェクト概要
このプロジェクトの概要は以下の通りです:
- RuboCopのすべてのLayoutといくつかの主要なLintをRustで再実装します
- RuboCopのパフォーマンスに関する問題を"部分的に"に解決することが最大の目的です
- RuboCopのすべての機能をRustで再実装することが目的ではありません
- RuboCopに追従するために、機能は少ないほうが良いと考えています
- 関心の中心はLayoutであり、Lintは主要なものに限定して提供します
- RuboCopとのAPI互換は必須です
- RuboCopの設計や実装について互換性を保つ必要はありません(Copのロジックは踏襲します)
- CLIツールとして開発します(gemでの提供は将来的に検討します)
### 数値目標
- RuboCopのサーバモードと比較して少なくとも40倍高速であることを目指します
- RuboCopのCIを20倍程度高速化することを目指します
  - RuboCopの前処理として実行することでRuboCopの実行時間を短縮する想定です
- LayoutはRuboCopの実行結果と100%同じ結果になることを期待します
- LintはRuboCopの実行結果と60%-80%程度同じ結果になることを期待します(主要なLintを10-20個実装すれば達成できる想定です)
### 開発方針
このプロジェクトの開発における方針は以下の通りです:
- 優先順位は以下の通りです
  1. RuboCopとのAPI互換性(既存のRuboCopユーザを取り込みたい)
  2. RuboCop独特の競合解決のロジック(パフォーマンスを犠牲にしても再現する必要があります)
  3. パフォーマンス(パフォーマンスは最大の優位性であるため非常に重要です)
  4. RuboCopの設計や実装の踏襲(まったく重要ではありません)
  5. 最適化(キャッシュや並列化やその他の最適化は最終的な課題です)
- 基本的な設計はRuffを参考にします(RuboCopは局所的に参照する程度にとどめます)
- パーサはRuby純正のPrismを使用します(Rustバインディングを使用します)
## 構成図
┌─────────────────────────────────────────────────────────────────────────────┐
│                            reukocyte_checker                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ check(source: &[u8]) → Vec<Diagnostic>                              │    │
│  │   1. ruby_prism::parse(source)                                      │    │
│  │   2. checker.visit(&ast)        ← AST-based rules (Lint)            │    │
│  │   3. trailing_whitespace::check ← Line-based rules (Layout)         │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                           │                                                 │
│                           ▼                                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ Checker                                                             │    │
│  │ ├── source: &[u8]                                                   │    │
│  │ ├── diagnostics: Vec<Diagnostic>                                    │    │
│  │ ├── line_index: LineIndex                                           │    │
│  │ └── call_visitor: CallVisitor ← analyze/call.rs                     │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                           │                                                 │
│                           ▼                                                 │
│  ┌──────────────────────┐    ┌──────────────────────────────────────────┐   │
│  │ Diagnostic           │    │ RuleId (enum)                            │   │
│  │ ├── rule: RuleId     │    │ ├── Layout(LayoutRule)                   │   │
│  │ ├── message: String  │    │ │   └── TrailingWhitespace               │   │
│  │ ├── severity: ...    │    │ └── Lint(LintRule)                       │   │
│  │ ├── start/end: usize │    │     └── Debugger                         │   │
│  │ └── fix: Option<Fix> │    ├── conflicts_with() → &[RuleId]           │   │
│  └──────────────────────┘    └── has_conflict_with(RuleId) → bool       │   │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                         Fix Application Pipeline                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ apply_fixes_with_loop_detection(source, diagnostics, unsafe_fixes)  │    │
│  │   → Result<(Vec<u8>, usize), InfiniteCorrectionLoop>                │    │
│  │                                                                     │    │
│  │   ┌───────────────────────────────────────────────────────────┐     │    │
│  │   │  Loop (max 200 iterations)                                │     │    │
│  │   │  ├── checksum(source) → detect infinite loops             │     │    │
│  │   │  ├── ConflictRegistry::new()                              │     │    │
│  │   │  ├── for each diagnostic with fix:                        │     │    │
│  │   │  │   ├── registry.conflicts_with_applied(rule_id)?        │     │    │
│  │   │  │   ├── corrector.merge(fix)?                            │     │    │
│  │   │  │   └── registry.mark_applied(rule_id)                   │     │    │
│  │   │  ├── corrector.apply(source)                              │     │    │
│  │   │  └── re-check for remaining violations                    │     │    │
│  │   └───────────────────────────────────────────────────────────┘     │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                           │                                                 │
│           ┌───────────────┼───────────────┐                                 │
│           ▼               ▼               ▼                                 │
│  ┌────────────────┐ ┌────────────────┐ ┌────────────────────────────────┐   │
│  │ Corrector      │ │ConflictRegistry│ │ ClobberingError                │   │
│  │ ├── merge()    │ │├── mark_applied│ │ ├── DifferentReplacements      │   │
│  │ └── apply()    │ │├── conflicts_  │ │ ├── SwallowedInsertion         │   │
│  │                │ ││   with_applied│ │ └── Overlapping                │   │
│  └────────────────┘ │└── clear()     │ └────────────────────────────────┘   │
│                     └────────────────┘                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                              Rules                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  rules/                                                                     │
│  ├── layout/                                                                │
│  │   └── trailing_whitespace.rs   ← Line-based, with Fix (Safe)             │
│  └── lint/                                                                  │
│      └── debugger.rs              ← AST-based (CallVisitor), no Fix         │
│                                                                             │
│  analyze/                                                                   │
│  └── call.rs                      ← CallVisitor for method call detection   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                              Data Flow                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Source Code                                                                │
│       │                                                                     │
│       ▼                                                                     │
│  ┌─────────┐    ┌──────────┐    ┌────────────────┐    ┌──────────────────┐  │
│  │  Parse  │───▶│  Check   │───▶│ Vec<Diagnostic>│───▶│ apply_fixes_with │  │
│  │ (Prism) │    │ (Rules)  │    │                │    │ _loop_detection  │  │
│  └─────────┘    └──────────┘    └────────────────┘    └──────────────────┘  │
│                                                                │            │
│                                                                ▼            │
│                                                        ┌──────────────┐     │
│                                                        │ Fixed Source │     │
│                                                        └──────────────┘     │
└─────────────────────────────────────────────────────────────────────────────┘
