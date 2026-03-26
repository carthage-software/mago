use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches;
use crate::rule::utils::consts::EXTENSION_FUNCTIONS;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct DisallowedFunctionsRule {
    meta: &'static RuleMeta,
    cfg: DisallowedFunctionsConfig,
}

/// An entry that can be either a simple string or an object with name and optional help.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum DisallowedEntry {
    /// Simple string entry (just the name).
    Simple(String),
    /// Entry with name and optional help message.
    WithHelp {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        help: Option<String>,
    },
}

impl DisallowedEntry {
    /// Returns the name of the disallowed item.
    pub fn name(&self) -> &str {
        match self {
            DisallowedEntry::Simple(name) => name,
            DisallowedEntry::WithHelp { name, .. } => name,
        }
    }

    /// Returns the custom help message, if any.
    pub fn help(&self) -> Option<&str> {
        match self {
            DisallowedEntry::Simple(_) => None,
            DisallowedEntry::WithHelp { help, .. } => help.as_deref(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct DisallowedFunctionsConfig {
    pub level: Level,
    #[serde(default)]
    pub functions: Vec<DisallowedEntry>,
    #[serde(default)]
    pub extensions: Vec<DisallowedEntry>,
}

impl Default for DisallowedFunctionsConfig {
    fn default() -> Self {
        Self { level: Level::Warning, functions: Vec::new(), extensions: Vec::new() }
    }
}

impl Config for DisallowedFunctionsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for DisallowedFunctionsRule {
    type Config = DisallowedFunctionsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Disallowed Functions",
            code: "disallowed-functions",
            description: indoc! {r"
                Flags calls to functions that are disallowed via rule configuration.

                You can specify which functions or extensions should be disallowed through the
                `functions` or `extensions` options. This helps enforce coding standards,
                security restrictions, or the usage of preferred alternatives.

                Each entry can be a simple string or an object with `name` and optional `help`:

                ```toml
                functions = [
                    'eval',
                    { name = 'error_log', help = 'Use MyLogger instead.' },
                ]
                ```
            "},
            good_example: indoc! {r"
                <?php

                function allowed_function(): void {
                    // ...
                }

                allowed_function(); // Not flagged
            "},
            bad_example: indoc! {r"
                <?php

                curl_init(); // Error: part of a disallowed extension
            "},
            category: Category::Security,

            requirements: RuleRequirements::None,
        };
        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionCall];
        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config.clone() }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::FunctionCall(function_call) = node else {
            return;
        };

        // Check explicit disallowed functions
        for entry in &self.cfg.functions {
            let function_name = entry.name();
            if !function_call_matches(ctx, function_call, function_name) {
                continue;
            }

            let help_text = entry.help().unwrap_or(
                "Use an alternative function or update your configuration if this restriction is no longer needed.",
            );

            let issue = Issue::new(self.cfg.level, format!("Function `{function_name}` is disallowed."))
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(function_call.span())
                        .with_message(format!("call to disallowed function `{function_name}`")),
                )
                .with_note(format!(
                    "The function `{function_name}` is explicitly disallowed by your project configuration."
                ))
                .with_help(help_text);

            ctx.collector.report(issue);

            return;
        }

        // Check disallowed extensions
        for (extension, functions) in &EXTENSION_FUNCTIONS {
            let Some(entry) = self.cfg.extensions.iter().find(|e| e.name().eq_ignore_ascii_case(extension)) else {
                continue;
            };

            let help_text = entry
                .help()
                .unwrap_or("Avoid using this extension or update your configuration if exceptions are acceptable.");

            for function_name in *functions {
                if !function_call_matches(ctx, function_call, function_name) {
                    continue;
                }

                ctx.collector.report(
                    Issue::new(
                        self.cfg.level,
                        format!("Function `{function_name}` from the `{extension}` extension is disallowed."),
                    )
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(function_call.span()).with_message(format!(
                        "call to `{function_name}`, which belongs to the disallowed `{extension}` extension"
                    )))
                    .with_note(format!(
                        "All functions from the `{extension}` extension are disallowed by your project configuration."
                    ))
                    .with_help(help_text),
                );

                return;
            }
        }
    }
}
