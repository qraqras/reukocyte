use regex::Regex;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// All node types for macro generation.
/// Each entry is the full type path (e.g., "ruby_prism::StatementsNode").
const ALL_NODE_TYPES: &[&str] = &[
    // ********** ruby_prism nodes **********
    "ruby_prism::AliasGlobalVariableNode",
    "ruby_prism::AliasMethodNode",
    "ruby_prism::AlternationPatternNode",
    "ruby_prism::AndNode",
    "ruby_prism::ArgumentsNode",
    "ruby_prism::ArrayNode",
    "ruby_prism::ArrayPatternNode",
    "ruby_prism::AssocNode",
    "ruby_prism::AssocSplatNode",
    "ruby_prism::BackReferenceReadNode",
    "ruby_prism::BeginNode",
    "ruby_prism::BlockArgumentNode",
    "ruby_prism::BlockLocalVariableNode",
    "ruby_prism::BlockNode",
    "ruby_prism::BlockParameterNode",
    "ruby_prism::BlockParametersNode",
    "ruby_prism::BreakNode",
    "ruby_prism::CallAndWriteNode",
    "ruby_prism::CallNode",
    "ruby_prism::CallOperatorWriteNode",
    "ruby_prism::CallOrWriteNode",
    "ruby_prism::CallTargetNode",
    "ruby_prism::CapturePatternNode",
    "ruby_prism::CaseMatchNode",
    "ruby_prism::CaseNode",
    "ruby_prism::ClassNode",
    "ruby_prism::ClassVariableAndWriteNode",
    "ruby_prism::ClassVariableOperatorWriteNode",
    "ruby_prism::ClassVariableOrWriteNode",
    "ruby_prism::ClassVariableReadNode",
    "ruby_prism::ClassVariableTargetNode",
    "ruby_prism::ClassVariableWriteNode",
    "ruby_prism::ConstantAndWriteNode",
    "ruby_prism::ConstantOperatorWriteNode",
    "ruby_prism::ConstantOrWriteNode",
    "ruby_prism::ConstantPathAndWriteNode",
    "ruby_prism::ConstantPathNode",
    "ruby_prism::ConstantPathOperatorWriteNode",
    "ruby_prism::ConstantPathOrWriteNode",
    "ruby_prism::ConstantPathTargetNode",
    "ruby_prism::ConstantPathWriteNode",
    "ruby_prism::ConstantReadNode",
    "ruby_prism::ConstantTargetNode",
    "ruby_prism::ConstantWriteNode",
    "ruby_prism::DefNode",
    "ruby_prism::DefinedNode",
    "ruby_prism::ElseNode",
    "ruby_prism::EmbeddedStatementsNode",
    "ruby_prism::EmbeddedVariableNode",
    "ruby_prism::EnsureNode",
    "ruby_prism::FalseNode",
    "ruby_prism::FindPatternNode",
    "ruby_prism::FlipFlopNode",
    "ruby_prism::FloatNode",
    "ruby_prism::ForNode",
    "ruby_prism::ForwardingArgumentsNode",
    "ruby_prism::ForwardingParameterNode",
    "ruby_prism::ForwardingSuperNode",
    "ruby_prism::GlobalVariableAndWriteNode",
    "ruby_prism::GlobalVariableOperatorWriteNode",
    "ruby_prism::GlobalVariableOrWriteNode",
    "ruby_prism::GlobalVariableReadNode",
    "ruby_prism::GlobalVariableTargetNode",
    "ruby_prism::GlobalVariableWriteNode",
    "ruby_prism::HashNode",
    "ruby_prism::HashPatternNode",
    "ruby_prism::IfNode",
    "ruby_prism::ImaginaryNode",
    "ruby_prism::ImplicitNode",
    "ruby_prism::ImplicitRestNode",
    "ruby_prism::InNode",
    "ruby_prism::IndexAndWriteNode",
    "ruby_prism::IndexOperatorWriteNode",
    "ruby_prism::IndexOrWriteNode",
    "ruby_prism::IndexTargetNode",
    "ruby_prism::InstanceVariableAndWriteNode",
    "ruby_prism::InstanceVariableOperatorWriteNode",
    "ruby_prism::InstanceVariableOrWriteNode",
    "ruby_prism::InstanceVariableReadNode",
    "ruby_prism::InstanceVariableTargetNode",
    "ruby_prism::InstanceVariableWriteNode",
    "ruby_prism::IntegerNode",
    "ruby_prism::InterpolatedMatchLastLineNode",
    "ruby_prism::InterpolatedRegularExpressionNode",
    "ruby_prism::InterpolatedStringNode",
    "ruby_prism::InterpolatedSymbolNode",
    "ruby_prism::InterpolatedXStringNode",
    "ruby_prism::ItLocalVariableReadNode",
    "ruby_prism::ItParametersNode",
    "ruby_prism::KeywordHashNode",
    "ruby_prism::KeywordRestParameterNode",
    "ruby_prism::LambdaNode",
    "ruby_prism::LocalVariableAndWriteNode",
    "ruby_prism::LocalVariableOperatorWriteNode",
    "ruby_prism::LocalVariableOrWriteNode",
    "ruby_prism::LocalVariableReadNode",
    "ruby_prism::LocalVariableTargetNode",
    "ruby_prism::LocalVariableWriteNode",
    "ruby_prism::MatchLastLineNode",
    "ruby_prism::MatchPredicateNode",
    "ruby_prism::MatchRequiredNode",
    "ruby_prism::MatchWriteNode",
    "ruby_prism::MissingNode",
    "ruby_prism::ModuleNode",
    "ruby_prism::MultiTargetNode",
    "ruby_prism::MultiWriteNode",
    "ruby_prism::NextNode",
    "ruby_prism::NilNode",
    "ruby_prism::NoKeywordsParameterNode",
    "ruby_prism::NumberedParametersNode",
    "ruby_prism::NumberedReferenceReadNode",
    "ruby_prism::OptionalKeywordParameterNode",
    "ruby_prism::OptionalParameterNode",
    "ruby_prism::OrNode",
    "ruby_prism::ParametersNode",
    "ruby_prism::ParenthesesNode",
    "ruby_prism::PinnedExpressionNode",
    "ruby_prism::PinnedVariableNode",
    "ruby_prism::PostExecutionNode",
    "ruby_prism::PreExecutionNode",
    "ruby_prism::ProgramNode",
    "ruby_prism::RangeNode",
    "ruby_prism::RationalNode",
    "ruby_prism::RedoNode",
    "ruby_prism::RegularExpressionNode",
    "ruby_prism::RequiredKeywordParameterNode",
    "ruby_prism::RequiredParameterNode",
    "ruby_prism::RescueModifierNode",
    "ruby_prism::RescueNode",
    "ruby_prism::RestParameterNode",
    "ruby_prism::RetryNode",
    "ruby_prism::ReturnNode",
    "ruby_prism::SelfNode",
    "ruby_prism::ShareableConstantNode",
    "ruby_prism::SingletonClassNode",
    "ruby_prism::SourceEncodingNode",
    "ruby_prism::SourceFileNode",
    "ruby_prism::SourceLineNode",
    "ruby_prism::SplatNode",
    "ruby_prism::StatementsNode",
    "ruby_prism::StringNode",
    "ruby_prism::SuperNode",
    "ruby_prism::SymbolNode",
    "ruby_prism::TrueNode",
    "ruby_prism::UndefNode",
    "ruby_prism::UnlessNode",
    "ruby_prism::UntilNode",
    "ruby_prism::WhenNode",
    "ruby_prism::WhileNode",
    "ruby_prism::XStringNode",
    "ruby_prism::YieldNode",
    // ********** custom nodes **********
    "crate::custom_nodes::AssignmentNode",
];

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let rules_dir = PathBuf::from(&manifest_dir).join("src/rules");

    // Collect all rule implementations
    let rule_impls = scan_rules(&rules_dir);

    // Generate the rule registry
    generate_registry(&out_dir, &rule_impls);

    // Tell Cargo to rerun if any rule file changes
    println!("cargo:rerun-if-changed=src/rules");
}

/// Scans all .rs files under the rules directory for `#[check(NodeType)]` attributes.
///
/// Returns a HashMap mapping node types to their implementing rules.
fn scan_rules(rules_dir: &Path) -> FxHashMap<String, Vec<RuleInfo>> {
    let mut node_to_rules: FxHashMap<String, Vec<RuleInfo>> = FxHashMap::default();
    // Track globally seen (module, name, node_key) tuples to avoid duplicates
    // across keys but allow the same rule to implement multiple node types.
    let mut seen: FxHashSet<(String, String, String)> = FxHashSet::default();

    // Pattern: #[check(NodeType)]
    // followed by: impl Check<NodeType<'_>> for RuleName
    // Match only node types (ending with Node). Allow optional lifetime (<'_>) in the generic.
    let check_pattern_node = Regex::new(r"#\[check\((\w+Node)\)\]\s*impl\s+Check<\w+(?:<'_>)?>\s+for\s+(\w+)").unwrap();
    // Pattern: #[check(Line)]
    // followed by: impl Check<Line> for RuleName
    let check_pattern_line = Regex::new(r"#\[check\(Line\)\]\s*impl\s+Check<Line(?:<'_>)?>\s+for\s+(\w+)").unwrap();
    // Pattern: #[check(File)]
    // followed by: pub fn check(checker: &mut Checker)
    let check_pattern_file = Regex::new(r"#\[check\(File\)\]").unwrap();

    // Walk through all .rs files in the rules directory
    for entry in walkdir(rules_dir) {
        if entry.extension().is_some_and(|ext| ext == "rs") {
            if let Ok(content) = fs::read_to_string(&entry) {
                // Find the module path for this rule
                let module_path = get_module_path(rules_dir, &entry);

                for cap in check_pattern_node.captures_iter(&content) {
                    let node_type = cap[1].to_string();
                    let rule_name = cap[2].to_string();

                    // Avoid duplicate entries for the same (module, rule_name, node_type)
                    if !seen.insert((module_path.clone(), rule_name.clone(), node_type.clone())) {
                        // Already saw this rule (maybe via another attribute); warn and skip
                        println!("cargo:warning=Duplicate rule registration skipped: {}::{}", module_path, rule_name);
                        continue;
                    }

                    node_to_rules.entry(node_type).or_default().push(RuleInfo {
                        name: rule_name,
                        module: module_path.clone(),
                    });
                }
                // Line-based rules
                for cap in check_pattern_line.captures_iter(&content) {
                    let rule_name = cap[1].to_string();
                    // Use special key "Line"
                    if !seen.insert((module_path.clone(), rule_name.clone(), "Line".to_string())) {
                        println!("cargo:warning=Duplicate rule registration skipped: {}::{}", module_path, rule_name);
                        continue;
                    }
                    node_to_rules.entry("Line".to_string()).or_default().push(RuleInfo {
                        name: rule_name,
                        module: module_path.clone(),
                    });
                }
                // File-based rules: look for attribute #[check(File)] anywhere in the file
                if check_pattern_file.is_match(&content) {
                    // Derive a rule name from the module path (last segment)
                    if !module_path.is_empty() {
                        let seg = module_path.rsplit("::").next().unwrap_or(&module_path);
                        let rule_name = seg.to_string();
                        if !seen.insert((module_path.clone(), rule_name.clone(), "File".to_string())) {
                            println!("cargo:warning=Duplicate rule registration skipped: {}::{}", module_path, rule_name);
                        } else {
                            node_to_rules.entry("File".to_string()).or_default().push(RuleInfo {
                                name: rule_name,
                                module: module_path.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    // Cross-check: ensure that no rule was registered under multiple different keys
    // (this likely indicates a mis-detection). We'll detect duplicates by scanning
    // the final map for repeated (module,name) across different keys.
    let mut seen_by_key: FxHashMap<(String, String), Vec<String>> = FxHashMap::default();
    for (key, vec) in &node_to_rules {
        for r in vec {
            seen_by_key.entry((r.module.clone(), r.name.clone())).or_default().push(key.clone());
        }
    }
    for ((module, name), keys) in &seen_by_key {
        if keys.len() > 1 {
            println!("cargo:warning=Rule {}::{} registered for multiple keys: {:?}", module, name, keys);
        }
    }

    node_to_rules
}

/// Simple recursive directory walker.
fn walkdir(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walkdir(&path));
            } else {
                files.push(path);
            }
        }
    }
    files
}

/// Determines the module path for a rule file relative to src/rules.
///
/// e.g., `src/rules/layout/indentation_width.rs` -> `layout::indentation_width`
fn get_module_path(rules_dir: &Path, file_path: &Path) -> String {
    let relative = file_path.strip_prefix(rules_dir).unwrap();
    let mut parts: Vec<&str> = relative.iter().filter_map(|p| p.to_str()).collect();

    // Remove the .rs extension from the last part
    if let Some(last) = parts.last_mut() {
        *last = last.strip_suffix(".rs").unwrap_or(last);
    }

    // Skip mod.rs files
    if parts.last() == Some(&"mod") {
        return String::new();
    }

    parts.join("::")
}

#[derive(Debug, Clone)]
struct RuleInfo {
    name: String,
    module: String,
}

impl RuleInfo {
    /// Get the config path for this rule to check if enabled.
    /// e.g., "layout::def_end_alignment" -> "layout.def_end_alignment.enabled"
    fn config_path(&self) -> String {
        // Convert module path to config path
        // e.g., "layout::def_end_alignment" -> "layout.def_end_alignment"
        self.module.replace("::", ".")
    }
}

/// Extracts the node name from a full type path.
/// e.g., "ruby_prism::CallNode" -> "CallNode"
fn node_name(type_path: &str) -> &str {
    type_path.rsplit("::").next().unwrap_or(type_path)
}

/// Generates the rule registry macro file.
fn generate_registry(out_dir: &str, rule_impls: &FxHashMap<String, Vec<RuleInfo>>) {
    let dest_path = Path::new(out_dir).join("rule_registry.rs");
    let mut file = File::create(&dest_path).unwrap();

    writeln!(file, "// Auto-generated by build.rs - DO NOT EDIT").unwrap();
    writeln!(file).unwrap();

    // Generate marker traits for node types with rules
    writeln!(file, "/// Marker traits for node types that have rules.").unwrap();
    writeln!(file, "pub mod has_rules {{").unwrap();
    for type_path in ALL_NODE_TYPES {
        let name = node_name(type_path);
        if rule_impls.contains_key(name) {
            let snake_name = to_snake_case(name);
            writeln!(file, "    /// `{}` has rules.", name).unwrap();
            writeln!(file, "    pub const {}: bool = true;", snake_name.to_uppercase()).unwrap();
        }
    }
    writeln!(file, "}}").unwrap();
    writeln!(file).unwrap();
    // Generate arrays of rule config paths grouped by category
    writeln!(file, "// AST rules (node-based)").unwrap();
    writeln!(file, "pub(crate) static __REUKO_AST_RULE_CONFIGS: &[&str] = &[").unwrap();
    for (node_key, rules) in rule_impls {
        if node_key == "Line" {
            continue;
        }
        for r in rules {
            writeln!(file, "    \"{}\",", r.config_path()).unwrap();
        }
    }
    writeln!(file, "];\n").unwrap();

    writeln!(file, "// Line-based rules (Line)").unwrap();
    writeln!(file, "pub(crate) static __REUKO_LINE_RULE_CONFIGS: &[&str] = &[").unwrap();
    if let Some(line_rules) = rule_impls.get("Line") {
        for r in line_rules {
            writeln!(file, "    \"{}\",", r.config_path()).unwrap();
        }
    }
    writeln!(file, "];\n").unwrap();

    writeln!(file, "// Token-based rules (placeholder)").unwrap();
    writeln!(file, "pub(crate) static __REUKO_TOKEN_RULE_CONFIGS: &[&str] = &[];\n").unwrap();
    writeln!(file).unwrap();
    writeln!(file, "// File-based rules").unwrap();
    writeln!(file, "pub(crate) static __REUKO_FILE_RULE_CONFIGS: &[&str] = &[").unwrap();
    if let Some(file_rules) = rule_impls.get("File") {
        for r in file_rules {
            writeln!(file, "    \"{}\",", r.config_path()).unwrap();
        }
    }
    writeln!(file, "];\n").unwrap();
    // Profiling registry for rules (used when RUEKO_PROFILE_RULES=1)
    writeln!(file, "use std::sync::{{Mutex, OnceLock}};").unwrap();
    writeln!(file, "use std::collections::HashMap;").unwrap();
    writeln!(
        file,
        "pub(crate) static __REUKO_PROFILE_RULES_REGISTRY: OnceLock<Mutex<HashMap<&'static str, (u128, u64)>>> = OnceLock::new();"
    )
    .unwrap();
    writeln!(file).unwrap();

    // Generate a macro for ALL node types (including those without rules)
    for type_path in ALL_NODE_TYPES {
        let name = node_name(type_path);
        let rules = rule_impls.get(name);
        let snake_name = to_snake_case(name);

        writeln!(file, "/// Runs all rules for `{}`.", name).unwrap();
        writeln!(file, "macro_rules! run_{}_rules {{", snake_name).unwrap();
        writeln!(file, "    ($node:expr, $checker:expr) => {{").unwrap();

        if let Some(rules) = rules {
            for rule in rules {
                let full_path = format!("crate::rules::{}::{}", rule.module, rule.name);
                let config_path = rule.config_path();
                // Generate enabled check and include/exclude check before calling the rule
                writeln!(file, "        {{").unwrap();
                writeln!(file, "            let cfg = &$checker.config().{};", config_path).unwrap();
                writeln!(
                    file,
                    "            if cfg.base.enabled && $checker.should_run_cop_cached(\"{}\", &cfg.base) {{",
                    config_path
                )
                .unwrap();
                writeln!(file, "                if std::env::var(\"RUEKO_PROFILE_RULES\").is_ok() {{").unwrap();
                writeln!(file, "                    let __reuko_rule_start = std::time::Instant::now();").unwrap();
                writeln!(
                    file,
                    "                    <{} as crate::rule::Check<{}<'_>>>::check($node, $checker);",
                    full_path, type_path
                )
                .unwrap();
                writeln!(
                    file,
                    "                    let __reuko_rule_dur = __reuko_rule_start.elapsed().as_micros() as u128;"
                )
                .unwrap();
                writeln!(
                    file,
                    "                    let __reuko_map = __REUKO_PROFILE_RULES_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));"
                )
                .unwrap();
                writeln!(file, "                    let mut __reuko_lock = __reuko_map.lock().unwrap();").unwrap();
                writeln!(
                    file,
                    "                    let e = __reuko_lock.entry(\"{}::{}\").or_insert((0u128, 0u64));",
                    rule.module, rule.name
                )
                .unwrap();
                writeln!(file, "                    e.0 += __reuko_rule_dur;").unwrap();
                writeln!(file, "                    e.1 += 1u64;").unwrap();
                writeln!(file, "                }} else {{").unwrap();
                writeln!(
                    file,
                    "                    <{} as crate::rule::Check<{}<'_>>>::check($node, $checker);",
                    full_path, type_path
                )
                .unwrap();
                writeln!(file, "                }}").unwrap();
                writeln!(file, "            }}").unwrap();
                writeln!(file, "        }}").unwrap();
            }
        }
        // Empty macro body for node types without rules

        writeln!(file, "    }};").unwrap();
        writeln!(file, "}}").unwrap();
        writeln!(file).unwrap();
    }
    // Generate a macro for line-based rules if any exist
    if let Some(line_rules) = rule_impls.get("Line") {
        writeln!(file, "/// Runs all line-based rules.").unwrap();
        writeln!(file, "macro_rules! run_line_rules {{").unwrap();
        writeln!(file, "    ($line:expr, $checker:expr) => {{").unwrap();

        for rule in line_rules {
            let full_path = format!("crate::rules::{}::{}", rule.module, rule.name);
            let config_path = rule.config_path();
            // For line rules, config path is the same as for layout rules
            writeln!(file, "        {{").unwrap();
            writeln!(file, "            let cfg = &$checker.config().{};", config_path).unwrap();
            writeln!(
                file,
                "            if cfg.base.enabled && $checker.should_run_cop_cached(\"{}\", &cfg.base) {{",
                config_path
            )
            .unwrap();
            writeln!(file, "                if std::env::var(\"RUEKO_PROFILE_RULES\").is_ok() {{").unwrap();
            writeln!(file, "                    let __reuko_rule_start = std::time::Instant::now();").unwrap();
            writeln!(
                file,
                "                    <{} as crate::rule::Check<crate::rule::Line<'_>>>::check($line, $checker);",
                full_path
            )
            .unwrap();
            writeln!(
                file,
                "                    let __reuko_rule_dur = __reuko_rule_start.elapsed().as_micros() as u128;"
            )
            .unwrap();
            writeln!(
                file,
                "                    let __reuko_map = __REUKO_PROFILE_RULES_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));"
            )
            .unwrap();
            writeln!(file, "                    let mut __reuko_lock = __reuko_map.lock().unwrap();").unwrap();
            writeln!(
                file,
                "                    let e = __reuko_lock.entry(\"{}::{}\").or_insert((0u128, 0u64));",
                rule.module, rule.name
            )
            .unwrap();
            writeln!(file, "                    e.0 += __reuko_rule_dur;").unwrap();
            writeln!(file, "                    e.1 += 1u64;").unwrap();
            writeln!(file, "                }} else {{").unwrap();
            writeln!(
                file,
                "                    <{} as crate::rule::Check<crate::rule::Line<'_>>>::check($line, $checker);",
                full_path
            )
            .unwrap();
            writeln!(file, "                }}").unwrap();
            writeln!(file, "            }}").unwrap();
            writeln!(file, "        }}").unwrap();
        }

        writeln!(file, "    }};\n}}").unwrap();
        writeln!(file).unwrap();
    }

    // Generate a macro for file-level rules (functions with `pub fn check(checker)`)
    if let Some(file_rules) = rule_impls.get("File") {
        writeln!(file, "/// Runs all file-level rules.").unwrap();
        writeln!(file, "macro_rules! run_file_rules {{").unwrap();
        writeln!(file, "    ($checker:expr) => {{").unwrap();
        for rule in file_rules {
            let full_path = format!("crate::rules::{}", rule.module);
            let config_path = rule.config_path();
            writeln!(file, "        {{").unwrap();
            writeln!(file, "            let cfg = &$checker.config().{};", config_path).unwrap();
            writeln!(
                file,
                "            if cfg.base.enabled && $checker.should_run_cop_cached(\"{}\", &cfg.base) {{",
                config_path
            )
            .unwrap();
            writeln!(file, "                if std::env::var(\"RUEKO_PROFILE_RULES\").is_ok() {{").unwrap();
            writeln!(file, "                    let __reuko_rule_start = std::time::Instant::now();").unwrap();
            writeln!(file, "                    {}::check($checker);", full_path).unwrap();
            writeln!(
                file,
                "                    let __reuko_rule_dur = __reuko_rule_start.elapsed().as_micros() as u128;"
            )
            .unwrap();
            writeln!(
                file,
                "                    let __reuko_map = __REUKO_PROFILE_RULES_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));"
            )
            .unwrap();
            writeln!(file, "                    let mut __reuko_lock = __reuko_map.lock().unwrap();").unwrap();
            writeln!(
                file,
                "                    let e = __reuko_lock.entry(\"{}::{}\").or_insert((0u128, 0u64));",
                rule.module, rule.name
            )
            .unwrap();
            writeln!(file, "                    e.0 += __reuko_rule_dur;").unwrap();
            writeln!(file, "                    e.1 += 1u64;").unwrap();
            writeln!(file, "                }} else {{").unwrap();
            writeln!(file, "                    {}::check($checker);", full_path).unwrap();
            writeln!(file, "                }}").unwrap();
            writeln!(file, "            }}").unwrap();
            writeln!(file, "        }}").unwrap();
        }
        writeln!(file, "    }};").unwrap();
        writeln!(file, "}}\n").unwrap();
        writeln!(file).unwrap();
    }

    // Generate a helper function to precompute should-run map for all rules.
    writeln!(file, "/// Precompute `should_run` for all rules and store into the checker's cache.").unwrap();
    writeln!(file, "pub fn __reuko_precompute_should_run_map(checker: &crate::checker::Checker<'_>) {{").unwrap();
    for (_key, vec) in rule_impls {
        for r in vec {
            let cfg_path = r.config_path();
            writeln!(file, "    {{").unwrap();
            writeln!(file, "        let cfg = &checker.config().{};", cfg_path).unwrap();
            writeln!(file, "        let enabled = cfg.base.enabled && checker.should_run_cop(&cfg.base);").unwrap();
            writeln!(file, "        checker.set_should_run_cached(\"{}\", enabled);", cfg_path).unwrap();
            writeln!(file, "    }}").unwrap();
        }
    }
    writeln!(file, "}}").unwrap();
}

/// Converts PascalCase to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}
