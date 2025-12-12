/// Define all cops and generate:
/// - `RubocopYaml` struct fields with serde rename attributes
/// - `from_rubocop_yaml` implementation for Config
/// - `merge_configs` function for configuration inheritance
///
/// # Usage
/// ```ignore
/// generate_rubocop_yaml! {
///     layout {
///         "Layout/EndAlignment" => EndAlignment, end_alignment,
///         "Layout/TrailingWhitespace" => TrailingWhitespace, trailing_whitespace,
///     }
///     lint {
///         "Lint/Debugger" => Debugger, debugger,
///     }
/// }
/// ```
macro_rules! generate_rubocop_yaml {
    (
        layout {
            $($layout_rename:literal => $layout_cop:ident, $layout_field:ident),* $(,)?
        }
        lint {
            $($lint_rename:literal => $lint_cop:ident, $lint_field:ident),* $(,)?
        }
    ) => {
        // ============================================================
        // RubocopYaml struct
        // ============================================================

        /// Root structure of a .rubocop.yml file.
        ///
        /// RuboCop YAML files have a flat structure where each top-level key is either:
        /// - A special key like `inherit_from`, `AllCops`, etc.
        /// - A cop name like `Layout/EndAlignment`, `Lint/Debugger`
        ///
        /// Each cop configuration is directly deserialized using `#[serde(rename)]`.
        #[derive(Debug, Clone, Default, serde::Deserialize)]
        pub struct RubocopYaml {
            #[serde(default)]
            pub inherit_from: InheritFrom,
            #[serde(rename = "AllCops", default)]
            pub all_cops: AllCopsConfig,
            $(
                #[serde(rename = $layout_rename, default)]
                pub $layout_field: super::layout::$layout_field::$layout_cop,
            )*
            $(
                #[serde(rename = $lint_rename, default)]
                pub $lint_field: super::lint::$lint_field::$lint_cop,
            )*
        }

        // ============================================================
        // Config::from_rubocop_yaml
        // ============================================================

        impl super::Config {
            /// Create a Config from a parsed RubocopYaml.
            pub fn from_rubocop_yaml(yaml: &RubocopYaml) -> Self {
                let mut cfg = super::Config {
                    all_cops: yaml.all_cops.clone(),
                    layout: super::layout::LayoutConfig {
                        $(
                            $layout_field: yaml.$layout_field.clone(),
                        )*
                    },
                    lint: super::lint::LintConfig {
                        $(
                            $lint_field: yaml.$lint_field.clone(),
                        )*
                    },
                };
                // Compile all AllCops and BaseCopConfig globsets to accelerate matches
                cfg.all_cops.compile_globs();
                $(
                    cfg.layout.$layout_field.base.compile_globs();
                )*
                $(
                    cfg.lint.$lint_field.base.compile_globs();
                )*
                cfg
            }
        }

        // ============================================================
        // merge_configs
        // ============================================================

        /// Merge two configurations. Child values override parent values.
        pub(super) fn merge_configs(parent: RubocopYaml, child: RubocopYaml) -> RubocopYaml {
            /// Merge a cop config: use child if it has explicit overrides.
            macro_rules! merge {
                ($parent:expr, $child:expr, $default:expr) => {{
                    if !$child.base.enabled && $default.base.enabled {
                        $child
                    } else if $child.base.enabled != $default.base.enabled
                        || $child.base.severity != $default.base.severity
                    {
                        $child
                    } else {
                        $child
                    }
                }};
            }

            RubocopYaml {
                inherit_from: child.inherit_from,
                all_cops: merge_all_cops(parent.all_cops, child.all_cops),
                $(
                    $layout_field: merge!(
                        parent.$layout_field,
                        child.$layout_field,
                        super::layout::$layout_field::$layout_cop::default()
                    ),
                )*
                $(
                    $lint_field: merge!(
                        parent.$lint_field,
                        child.$lint_field,
                        super::lint::$lint_field::$lint_cop::default()
                    ),
                )*
            }
        }
    };
}

pub(super) use generate_rubocop_yaml;
