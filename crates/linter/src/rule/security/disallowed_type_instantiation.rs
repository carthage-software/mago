use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct DisallowedTypeInstantiationRule {
    meta: &'static RuleMeta,
    cfg: DisallowedTypeInstantiationConfig,
}

/// An entry that can be either a simple string or an object with name and optional help.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum DisallowedTypeEntry {
    /// Simple string entry (just the name).
    Simple(String),
    /// Entry with name and optional help message.
    WithHelp {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        help: Option<String>,
    },
}

impl DisallowedTypeEntry {
    /// Returns the name of the disallowed type.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            DisallowedTypeEntry::Simple(name) => name,
            DisallowedTypeEntry::WithHelp { name, .. } => name,
        }
    }

    /// Returns the custom help message, if any.
    #[must_use]
    pub fn help(&self) -> Option<&str> {
        match self {
            DisallowedTypeEntry::Simple(_) => None,
            DisallowedTypeEntry::WithHelp { help, .. } => help.as_deref(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct DisallowedTypeInstantiationConfig {
    pub level: Level,
    #[serde(default)]
    pub types: Vec<DisallowedTypeEntry>,
}

impl Default for DisallowedTypeInstantiationConfig {
    fn default() -> Self {
        Self { level: Level::Warning, types: Vec::new() }
    }
}

impl Config for DisallowedTypeInstantiationConfig {
    fn level(&self) -> Level {
        self.level
    }

    fn default_enabled() -> bool {
        false
    }
}

impl LintRule for DisallowedTypeInstantiationRule {
    type Config = DisallowedTypeInstantiationConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Disallowed Type Instantiation",
            code: "disallowed-type-instantiation",
            description: indoc! {r"
                Flags direct instantiation of specific types that are disallowed via rule configuration.

                This rule helps enforce architectural patterns such as factory methods or provider patterns
                by preventing direct instantiation of specific classes. This is useful for ensuring consistent
                configuration, centralizing object creation, and maintaining architectural boundaries.

                Each entry can be a simple string or an object with `name` and optional `help`:

                ```toml
                [linter.rules]
                disallowed-type-instantiation = {
                    enabled = true,
                    types = [
                        'HttpService\\Client',
                        { name = 'DatabaseConnection', help = 'Use DatabaseFactory::create() instead' },
                    ]
                }
                ```
            "},
            good_example: indoc! {r"
                <?php

                // Using factory pattern instead of direct instantiation
                $client = ClientProvider::getClient();
            "},
            bad_example: indoc! {r"
                <?php

                // Direct instantiation of disallowed type
                $client = new HttpService\Client();

                // Another disallowed instantiation
                $db = new DatabaseConnection('localhost', 'user', 'pass');
            "},
            category: Category::Security,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Instantiation];
        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config.clone() }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Instantiation(instantiation) = node else {
            return;
        };

        let Expression::Identifier(identifier) = instantiation.class else {
            return;
        };

        let class_name = ctx.lookup_name(identifier);

        for entry in &self.cfg.types {
            let disallowed_type = entry.name();
            if !class_name.eq_ignore_ascii_case(disallowed_type) {
                continue;
            }

            let help_text = entry.help().unwrap_or(
                "Use an alternative factory or provider pattern, or update your configuration if this restriction is no longer needed.",
            );

            let issue = Issue::new(
                self.cfg.level,
                format!("Direct instantiation of type `{disallowed_type}` is disallowed."),
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(instantiation.span())
                    .with_message(format!("direct instantiation of disallowed type `{disallowed_type}`")),
            )
            .with_note(format!(
                "The type `{disallowed_type}` is explicitly disallowed from being instantiated directly by your project configuration."
            ))
            .with_help(help_text);

            ctx.collector.report(issue);

            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = allowed_types_not_flagged,
        rule = DisallowedTypeInstantiationRule,
        code = indoc! {r"
            <?php

            $dateTime = new DateTime();
            $stdClass = new stdClass();
            $exception = new Exception('error');
        "}
    }

    test_lint_success! {
        name = different_class_same_short_name,
        rule = DisallowedTypeInstantiationRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.disallowed_type_instantiation.config.types = vec![
                DisallowedTypeEntry::Simple("HttpService\\Client".to_string()),
            ];
        },
        code = indoc! {r"
            <?php

            namespace App;

            class Client {}

            // This should NOT be flagged (different class than HttpService\Client)
            $myClient = new Client();
        "}
    }

    test_lint_failure! {
        name = disallow_type_simple_string,
        rule = DisallowedTypeInstantiationRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.disallowed_type_instantiation.config.types = vec![
                DisallowedTypeEntry::Simple("HttpService\\Client".to_string()),
            ];
        },
        code = indoc! {r"
            <?php

            use HttpService\Client;

            $client = new Client();
        "}
    }

    test_lint_failure! {
        name = disallow_type_with_help,
        rule = DisallowedTypeInstantiationRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.disallowed_type_instantiation.config.types = vec![
                DisallowedTypeEntry::WithHelp {
                    name: "HttpService\\Client".to_string(),
                    help: Some("Use ClientProvider::getClient() instead".to_string()),
                },
            ];
        },
        code = indoc! {r"
            <?php

            use HttpService\Client;

            $client = new Client();
        "}
    }

    test_lint_failure! {
        name = disallow_fully_qualified_name,
        rule = DisallowedTypeInstantiationRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.disallowed_type_instantiation.config.types = vec![
                DisallowedTypeEntry::Simple("HttpService\\Client".to_string()),
            ];
        },
        code = indoc! {r"
            <?php

            $client = new \HttpService\Client();
        "}
    }

    test_lint_failure! {
        name = case_insensitive_matching,
        rule = DisallowedTypeInstantiationRule,
        count = 3,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.disallowed_type_instantiation.config.types = vec![
                DisallowedTypeEntry::Simple("HttpService\\Client".to_string()),
            ];
        },
        code = indoc! {r"
            <?php

            use HttpService\Client;

            $a = new Client();
            $b = new client();
            $c = new CLIENT();
        "}
    }

    test_lint_failure! {
        name = multiple_disallowed_types,
        rule = DisallowedTypeInstantiationRule,
        count = 2,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.disallowed_type_instantiation.config.types = vec![
                DisallowedTypeEntry::Simple("HttpService\\Client".to_string()),
                DisallowedTypeEntry::WithHelp {
                    name: "DatabaseConnection".to_string(),
                    help: Some("Use DatabaseFactory::create() instead".to_string()),
                },
            ];
        },
        code = indoc! {r"
            <?php

            use HttpService\Client;

            $client = new Client();
            $db = new \DatabaseConnection('localhost', 'user', 'pass');
        "}
    }

    test_lint_failure! {
        name = disallow_in_namespace_context,
        rule = DisallowedTypeInstantiationRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.disallowed_type_instantiation.config.types = vec![
                DisallowedTypeEntry::Simple("HttpService\\Client".to_string()),
            ];
        },
        code = indoc! {r"
            <?php

            namespace App\Services;

            use HttpService\Client;

            class MyService {
                public function create() {
                    return new Client();
                }
            }
        "}
    }

    test_lint_failure! {
        name = disallow_with_constructor_arguments,
        rule = DisallowedTypeInstantiationRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.disallowed_type_instantiation.config.types = vec![
                DisallowedTypeEntry::Simple("DatabaseConnection".to_string()),
            ];
        },
        code = indoc! {r"
            <?php

            $db = new \DatabaseConnection('localhost', 'user', 'password');
        "}
    }

    test_lint_failure! {
        name = disallow_without_parentheses,
        rule = DisallowedTypeInstantiationRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.disallowed_type_instantiation.config.types = vec![
                DisallowedTypeEntry::Simple("stdClass".to_string()),
            ];
        },
        code = indoc! {r"
            <?php

            $obj = new stdClass;
        "}
    }
}
