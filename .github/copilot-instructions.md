# Reukocyte

## お願い
- [MUST]特に指定がない限り日本語で回答してください
- [MUST]絶対に勝手にコードの修正を行わないでください
- [MUST]コードの修正前に方針を説明して必ず了承を得てください
- [SHOULD]処理の流れは図示してください

## 直近のタスク
このプロジェクトの直近のタスクは以下の通りです:
- 大規模ファイルでのパフォーマンスがRuboCop以下になるので改善する
  - SementicModelを廃止
  - パフォーマンス優先で設計を見直す

- Layout/*の実装
- LSP実装
- Reukecyte専用の設定ファイルの設計と実装

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
- Ruffのような開発体験を提供するためにLSPの一部の機能を実装します(Diagnostics、Formatting、CodeActionsのみ)
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
  4. LSP実装(Diagnostics、Formatting、CodeActionsのみで、残りはRubyLSPに任せます)(Ruffのような開発体験を提供するために非常に重要です)
  5. RuboCopの設計や実装の踏襲(まったく重要ではありません)
  6. 最適化(キャッシュや並列化やその他の最適化は最終的な課題です)
- 基本的な設計はRuffを参考にします(RuboCopは局所的に参照する程度にとどめます)
- パーサはRuby純正のPrismを使用します(Rustバインディングを使用します)
## 構成図

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              crates/                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│  reukocyte/           ← CLI バイナリ (rueko)                                │
│  reukocyte_checker/   ← コアライブラリ                                       │
│  reukocyte_macros/    ← proc-macro (将来用)                                 │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                         reukocyte_checker                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ check(source, config, file_path) → Vec<Diagnostic>                  │    │
│  │   1. ruby_prism::parse(source)                                      │    │
│  │   2. checker.build_index(&ast)  ← SemanticModel構築                 │    │
│  │   3. checker.visit_nodes(&ast)        ← AST-based rules                   │    │
│  │   4. run_line_rules!()          ← Line-based rules (マクロ生成)      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                              Checker                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ Checker<'rk>                                                        │    │
│  │ ├── source: &[u8]                                                   │    │
│  │ ├── config: &Config                                                 │    │
│  │ ├── file_path: Option<&str>     ← Include/Exclude用                 │    │
│  │ ├── ignored_nodes: FxHashSet    ← 重複検出回避用                     │    │
│  │ ├── line_index: LineIndex       ← offset↔行列変換                   │    │
│  │ ├── raw_diagnostics: Vec<RawDiagnostic>                             │    │
│  │ └── semantic: SemanticModel     ← AST親子関係・スコープ               │    │
│  ├─────────────────────────────────────────────────────────────────────┤    │
│  │ メソッド                                                             │    │
│  │ ├── should_run_cop(include, exclude) → bool                         │    │
│  │ │   ├── is_file_included()  ← globパターンマッチ                     │    │
│  │ │   └── is_file_excluded()  ← globパターンマッチ                     │    │
│  │ ├── add_diagnostic()                                                │    │
│  │ └── ignore_node() / is_node_ignored()                               │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                          Config (RuboCop互換)                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ Config                                                              │    │
│  │ ├── all_cops: AllCopsConfig                                         │    │
│  │ │   ├── exclude: Vec<String>   ← グローバルExclude                   │    │
│  │ │   └── target_ruby_version    ← (将来用)                           │    │
│  │ ├── layout: LayoutConfig                                            │    │
│  │ │   ├── trailing_whitespace: TrailingWhitespace                     │    │
│  │ │   ├── empty_lines: EmptyLines                                     │    │
│  │ │   ├── leading_empty_lines: LeadingEmptyLines                      │    │
│  │ │   ├── trailing_empty_lines: TrailingEmptyLines                    │    │
│  │ │   ├── indentation_style: IndentationStyle                         │    │
│  │ │   ├── indentation_width: IndentationWidth                         │    │
│  │ │   ├── indentation_consistency: IndentationConsistency             │    │
│  │ │   ├── end_alignment: EndAlignment                                 │    │
│  │ │   ├── def_end_alignment: DefEndAlignment                          │    │
│  │ │   └── begin_end_alignment: BeginEndAlignment                      │    │
│  │ └── lint: LintConfig                                                │    │
│  │     └── debugger: Debugger                                          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ BaseCopConfig (#[serde(flatten)]で各Copに埋め込み)                   │    │
│  │ ├── enabled: bool                                                   │    │
│  │ ├── severity: Severity                                              │    │
│  │ ├── exclude: Vec<String>   ← Cop固有Exclude                         │    │
│  │ └── include: Vec<String>   ← Cop固有Include                         │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  config/                                                                    │
│  ├── base.rs          ← BaseCopConfig定義                                   │
│  ├── loader.rs        ← .rubocop.yml読込・inherit_from解決                  │
│  ├── macros.rs        ← define_cops!マクロ (Config構築)                     │
│  ├── yaml.rs          ← RubocopYaml, AllCopsConfig                          │
│  ├── serde_helpers.rs ← Enabled/Severity deserialize                       │
│  ├── layout/          ← Layout各Copの設定struct (11ファイル)                │
│  └── lint/            ← Lint各Copの設定struct (1ファイル)                   │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                             Rules (11 Cops)                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  rules/                                                                     │
│  ├── layout/  (10 rules)                                                    │
│  │   ├── trailing_whitespace.rs    ← Line-based, Fix ✓                      │
│  │   ├── empty_lines.rs            ← Line-based, Fix ✓                      │
│  │   ├── leading_empty_lines.rs    ← Line-based, Fix ✓                      │
│  │   ├── trailing_empty_lines.rs   ← Line-based, Fix ✓                      │
│  │   ├── indentation_style.rs      ← Line-based, Fix ✓                      │
│  │   ├── indentation_width.rs      ← AST-based, Fix ✓                       │
│  │   ├── indentation_consistency.rs← AST-based, Fix ✓                       │
│  │   ├── end_alignment.rs          ← AST-based, Fix ✓                       │
│  │   ├── def_end_alignment.rs      ← AST-based, Fix ✓                       │
│  │   └── begin_end_alignment.rs    ← AST-based, Fix ✓                       │
│  └── lint/    (1 rule)                                                      │
│      └── debugger.rs               ← AST-based, No Fix                      │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                           Rule System                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────────────┐    ┌──────────────────────────────────────────┐   │
│  │ RuleId (enum)        │    │ Diagnostic                               │   │
│  │ ├── Layout(...)      │    │ ├── rule: RuleId                         │   │
│  │ │   ├── BeginEnd...  │    │ ├── message: String                      │   │
│  │ │   ├── DefEnd...    │    │ ├── severity: Severity                   │   │
│  │ │   ├── EmptyLines   │    │ ├── start/end: usize                     │   │
│  │ │   ├── EndAlignment │    │ └── fix: Option<Fix>                     │   │
│  │ │   ├── Indentation..│    └──────────────────────────────────────────┘   │
│  │ │   ├── Leading...   │                                                   │
│  │ │   ├── Trailing...  │    ┌──────────────────────────────────────────┐   │
│  │ │   └── TrailingWS   │    │ Check<N> trait                           │   │
│  │ └── Lint(...)        │    │   fn check(node: &N, checker: &mut ...)  │   │
│  │     └── Debugger     │    │   ← 各ノード型に対してルールが実装        │   │
│  └──────────────────────┘    └──────────────────────────────────────────┘   │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                      Fix Application Pipeline                               │
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
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                           Semantic Model                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  semantic/                                                                  │
│  ├── mod.rs                                                                 │
│  ├── model.rs    ← SemanticModel (ノードスタック管理)                        │
│  └── nodes.rs    ← NodeId (ノード識別用)                                    │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                              Utilities                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ├── locator.rs     ← LineIndex (offset↔行列変換)                           │
│  ├── corrector.rs   ← Corrector (Fix適用・マージ)                           │
│  ├── conflict.rs    ← ConflictRegistry (ルール競合管理)                      │
│  ├── diagnostic.rs  ← Diagnostic, RawDiagnostic, Severity, Fix              │
│  ├── fix.rs         ← Fix構築ヘルパー                                       │
│  └── custom_nodes/  ← AssignmentNode等カスタムノード                        │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                        Code Generation (build.rs)                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  build.rs → OUT_DIR/rule_registry.rs                                        │
│  ├── run_ast_rules!()    ← AST訪問時のルールディスパッチ                     │
│  └── run_line_rules!()   ← 行ベースルールのディスパッチ                      │
│                                                                             │
│  生成コード例:                                                               │
│  if cfg.base.enabled && $checker.should_run_cop(&cfg.base.include, ...) {   │
│      RuleName::check(node, $checker);                                       │
│  }                                                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                              Data Flow                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  .rubocop.yml ──────────────────────────────────────────┐                   │
│       │                                                 │                   │
│       ▼                                                 ▼                   │
│  ┌──────────────┐                              ┌──────────────┐             │
│  │ load_rubocop │ ← inherit_from解決            │   Config     │             │
│  │    _yaml()   │ ← マージ処理                  │  (Runtime)   │             │
│  └──────────────┘                              └──────────────┘             │
│                                                        │                    │
│  Source Code (.rb)                                     │                    │
│       │                                                │                    │
│       ▼                                                ▼                    │
│  ┌─────────┐    ┌──────────────┐    ┌────────────────────────────────────┐  │
│  │  Parse  │───▶│   Checker    │───▶│         Vec<Diagnostic>            │  │
│  │ (Prism) │    │  (Rules実行)  │    │ (violations + optional fixes)      │  │
│  └─────────┘    └──────────────┘    └────────────────────────────────────┘  │
│                        │                              │                     │
│                        │                              ▼                     │
│  File Path ────────────┘                  ┌────────────────────────────┐    │
│  (Include/Exclude判定用)                   │ apply_fixes_with_loop_     │    │
│                                           │ detection() [if -a flag]   │    │
│                                           └────────────────────────────┘    │
│                                                       │                     │
│                                                       ▼                     │
│                                              ┌──────────────┐               │
│                                              │ Fixed Source │               │
│                                              └──────────────┘               │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                          Performance (実測値)                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  単一ファイル (4,700行)                                                      │
│  ├── Reukocyte:           ~6ms   (基準)                                     │
│  ├── RuboCop (server):  ~250ms   (42倍遅い)                                 │
│  └── RuboCop (normal):  ~400ms   (67倍遅い)                                 │
│                                                                             │
│  複数ファイル (50ファイル)                                                   │
│  ├── Reukocyte:           ~4ms   (基準)                                     │
│  └── RuboCop (server):  ~275ms   (69倍遅い)                                 │
│                                                                             │
│  → 目標の「サーバモード比40倍」を達成 ✓                                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```
