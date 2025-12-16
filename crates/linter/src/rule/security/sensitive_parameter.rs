use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
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
use crate::rule::utils::security::is_password;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct SensitiveParameterRule {
    meta: &'static RuleMeta,
    cfg: SensitiveParameterConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct SensitiveParameterConfig {
    pub level: Level,
}

impl Default for SensitiveParameterConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for SensitiveParameterConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for SensitiveParameterRule {
    type Config = SensitiveParameterConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Sensitive Parameter",
            code: "sensitive-parameter",
            description: indoc! {r"
                Requires that parameters that are likely to contain sensitive information (e.g., passwords)
                are marked with the `#[SensitiveParameter]` attribute to prevent accidental logging or exposure.

                This rule only applies to PHP 8.2 and later, as the `SensitiveParameter` attribute was introduced in PHP 8.2.
            "},
            good_example: indoc! {r"
                <?php

                function login(string $username, #[SensitiveParameter] string $password): void {
                   // ...
                }
            "},
            bad_example: indoc! {r"
                <?php

                function login(string $username, string $password): void {
                   // ...
                }
            "},
            category: Category::Security,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP82)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionLikeParameter];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::FunctionLikeParameter(parameter) = node else {
            return;
        };

        if !is_password(parameter.variable.name) {
            return; // Not a password-related parameter, no issue
        }

        for attribute_list in &parameter.attribute_lists {
            for attribute in &attribute_list.attributes {
                let name = ctx.resolved_names.get(&attribute.name);

                if name.eq_ignore_ascii_case("SensitiveParameter") {
                    return; // Attribute found, no issue
                }
            }
        }

        let issue = Issue::new(self.cfg.level(), "Parameters that may contain sensitive information should be marked with the `#[SensitiveParameter]` attribute.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(parameter.variable.span).with_message("Sensitive parameter found here."))
            .with_note("Marking sensitive parameters helps prevent accidental logging or exposure of sensitive data in exception backtraces.")
            .with_help("Add the `#[SensitiveParameter]` attribute to the parameter declaration.");

        ctx.collector.propose(issue, |plan| {
            let start_position = parameter.start_position();

            plan.insert(start_position.offset, "#[\\SensitiveParameter] ", SafetyClassification::Safe);
        });
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use mago_php_version::PHPVersion;

    use super::SensitiveParameterRule;
    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = password_with_sensitive_parameter_attribute,
        rule = SensitiveParameterRule,
        settings = |s: &mut Settings| s.php_version = PHPVersion::PHP82,
        code = indoc! {r"
            <?php

            function login(string $username, #[SensitiveParameter] string $password): void {
                // ...
            }
        "}
    }

    test_lint_success! {
        name = password_with_fqn_sensitive_parameter_attribute,
        rule = SensitiveParameterRule,
        settings = |s: &mut Settings| s.php_version = PHPVersion::PHP82,
        code = indoc! {r"
            <?php

            function login(string $username, #[\SensitiveParameter] string $password): void {
                // ...
            }
        "}
    }

    test_lint_success! {
        name = non_password_parameter_without_attribute,
        rule = SensitiveParameterRule,
        settings = |s: &mut Settings| s.php_version = PHPVersion::PHP82,
        code = indoc! {r"
            <?php

            function login(string $username, string $email): void {
                // ...
            }
        "}
    }

    test_lint_failure! {
        name = password_without_attribute,
        rule = SensitiveParameterRule,
        settings = |s: &mut Settings| s.php_version = PHPVersion::PHP82,
        code = indoc! {r"
            <?php

            function login(string $username, string $password): void {
                // ...
            }
        "}
    }

    test_lint_failure! {
        name = multiple_password_params_without_attribute,
        rule = SensitiveParameterRule,
        count = 2,
        settings = |s: &mut Settings| s.php_version = PHPVersion::PHP82,
        code = indoc! {r"
            <?php

            function changePassword(string $oldPassword, string $newPassword): void {
                // ...
            }
        "}
    }

    test_lint_failure! {
        name = password_without_attribute_on_php83,
        rule = SensitiveParameterRule,
        settings = |s: &mut Settings| s.php_version = PHPVersion::PHP83,
        code = indoc! {r"
            <?php

            function login(string $username, string $password): void {
                // ...
            }
        "}
    }

    test_lint_failure! {
        name = password_without_attribute_on_php84,
        rule = SensitiveParameterRule,
        settings = |s: &mut Settings| s.php_version = PHPVersion::PHP84,
        code = indoc! {r"
            <?php

            function login(string $username, string $password): void {
                // ...
            }
        "}
    }
}
