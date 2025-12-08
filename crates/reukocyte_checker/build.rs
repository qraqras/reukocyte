//! Build script for automatic rule registry generation.
//!
//! This script scans all rule files in `src/rules/` and generates macros
//! that dispatch rules for each node type, enabling static dispatch.

use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// All node types for macro generation.
/// Each entry is the full type path (e.g., "ruby_prism::StatementsNode").
const ALL_NODE_TYPES: &[&str] = &[
    // ruby_prism nodes
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
    // Custom wrapper types
    "crate::utility::assignment::AssignmentNode",
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

/// Scans all .rs files under the rules directory for `impl Check<NodeType<'_>> for RuleName`.
///
/// Returns a HashMap mapping node types to their implementing rules.
fn scan_rules(rules_dir: &Path) -> HashMap<String, Vec<RuleInfo>> {
    let mut node_to_rules: HashMap<String, Vec<RuleInfo>> = HashMap::new();

    // Pattern: impl Check<NodeType<'_>> for RuleName
    let check_pattern = Regex::new(r"impl\s+Check<(\w+)<'_>>\s+for\s+(\w+)").unwrap();

    // Walk through all .rs files in the rules directory
    for entry in walkdir(rules_dir) {
        if entry.extension().is_some_and(|ext| ext == "rs") {
            if let Ok(content) = fs::read_to_string(&entry) {
                // Find the module path for this rule
                let module_path = get_module_path(rules_dir, &entry);

                for cap in check_pattern.captures_iter(&content) {
                    let node_type = cap[1].to_string();
                    let rule_name = cap[2].to_string();

                    node_to_rules.entry(node_type).or_default().push(RuleInfo {
                        name: rule_name,
                        module: module_path.clone(),
                    });
                }
            }
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

/// Extracts the node name from a full type path.
/// e.g., "ruby_prism::CallNode" -> "CallNode"
fn node_name(type_path: &str) -> &str {
    type_path.rsplit("::").next().unwrap_or(type_path)
}

/// Generates the rule registry macro file.
fn generate_registry(out_dir: &str, rule_impls: &HashMap<String, Vec<RuleInfo>>) {
    let dest_path = Path::new(out_dir).join("rule_registry.rs");
    let mut file = File::create(&dest_path).unwrap();

    writeln!(file, "// Auto-generated by build.rs - DO NOT EDIT").unwrap();
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
                writeln!(file, "        if $checker.is_enabled(<{} as crate::rule::Rule>::ID) {{", full_path).unwrap();
                writeln!(
                    file,
                    "            <{} as crate::rule::Check<{}<'_>>>::check($node, $checker);",
                    full_path, type_path
                )
                .unwrap();
                writeln!(file, "        }}").unwrap();
            }
        }
        // Empty macro body for node types without rules

        writeln!(file, "    }};").unwrap();
        writeln!(file, "}}").unwrap();
        writeln!(file).unwrap();
    }
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
