use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

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
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct DeprecatedShellExecuteStringRule {
    meta: &'static RuleMeta,
    cfg: DeprecatedShellExecuteStringConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct DeprecatedShellExecuteStringConfig {
    pub level: Level,
}

impl Default for DeprecatedShellExecuteStringConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for DeprecatedShellExecuteStringConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for DeprecatedShellExecuteStringRule {
    type Config = DeprecatedShellExecuteStringConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Deprecated Shell Execute String",
            code: "deprecated-shell-execute-string",
            description: indoc! {"
                Detect the usage of deprecated shell execute strings in PHP code.

                In PHP 8.5, the shell execute string syntax (enclosed in backticks, e.g., `` `ls -l` ``) has been deprecated.

                This rule identifies instances of shell execute strings and provides guidance on how to replace them with safer alternatives,
                such as using the `shell_exec()` function or other appropriate methods for executing shell commands.
            "},
            good_example: indoc! {r"
                <?php

                shell_exec('ls -l');
            "},
            bad_example: indoc! {r"
                <?php

                `ls -l`;
            "},
            category: Category::Deprecation,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP85)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::ShellExecuteString];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::ShellExecuteString(shell_execute_string) = node else {
            return;
        };

        let issue = Issue::new(self.cfg.level(), "Usage of deprecated shell execute string detected.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(shell_execute_string.span())
                    .with_message("This shell execute string is deprecated"),
            )
            .with_note("Shell execute strings (enclosed in backticks) have been deprecated in PHP 8.5 due to security concerns and potential vulnerabilities.")
            .with_note("It is recommended to replace them with safer alternatives, such as using the `shell_exec()` function or other appropriate methods for executing shell commands.")
            .with_help("Consider refactoring the code to use `shell_exec()` or similar functions instead of shell execute strings.")
            .with_link("https://wiki.php.net/rfc/deprecations_php_8_5");

        ctx.collector.report(issue);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = deprecated_shell_execute_string,
        rule = DeprecatedShellExecuteStringRule,
        code = indoc! {r"
            <?php

            `ls -l`;
        "}
    }

    test_lint_success! {
        name = shell_exec_not_flagged,
        rule = DeprecatedShellExecuteStringRule,
        code = indoc! {r"
            <?php

            shell_exec('ls -l');
        "}
    }
}
