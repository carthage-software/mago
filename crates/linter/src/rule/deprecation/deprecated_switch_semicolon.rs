use indoc::indoc;
use mago_syntax::ast::SwitchCaseSeparator;
use mago_text_edit::TextEdit;
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
pub struct DeprecatedSwitchSemicolonRule {
    meta: &'static RuleMeta,
    cfg: DeprecatedSwitchSemicolonConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct DeprecatedSwitchSemicolonConfig {
    pub level: Level,
}

impl Default for DeprecatedSwitchSemicolonConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for DeprecatedSwitchSemicolonConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for DeprecatedSwitchSemicolonRule {
    type Config = DeprecatedSwitchSemicolonConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Deprecated Switch Semicolon",
            code: "deprecated-switch-semicolon",
            description: indoc! {"
                Detect the usage of semicolon as a switch case separator.

                In PHP 8.5, the use of a semicolon (`;`) as a case separator in switch statements has been deprecated.

                Instead, the colon (`:`) should be used to separate case statements.
            "},
            good_example: indoc! {r"
                <?php

                switch ($value) {
                    case 1:
                        // code for case 1
                        break;
                    case 2:
                        // code for case 2
                        break;
                    default:
                        // default case
                        break;
                }
            "},
            bad_example: indoc! {r"
                <?php

                switch ($value) {
                    case 1;
                        // code for case 1
                        break;
                    case 2;
                        // code for case 2
                        break;
                    default;
                        // default case
                        break;
                }
            "},
            category: Category::Deprecation,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP85)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::SwitchCaseSeparator];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::SwitchCaseSeparator(SwitchCaseSeparator::SemiColon(semicolon)) = node else {
            return;
        };

        let issue = Issue::new(self.cfg.level(), format!("Usage of semicolon as a switch case separator is deprecated."))
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(semicolon.span())
                    .with_message("This semicolon is used as a switch case separator and is deprecated."),
            )
            .with_note("The use of a semicolon (`;`) as a case separator in switch statements has been deprecated since PHP 8.5.")
            .with_note("It is recommended to replace it with a colon (`:`) to separate case statements.")
            .with_help("Consider refactoring the switch statement to use colons (`:`) instead of semicolons (`;`) as case separators.")
            .with_link("https://wiki.php.net/rfc/deprecations_php_8_5");

        ctx.collector.propose(issue, |edits| {
            edits.push(TextEdit::replace(semicolon.span(), ":"));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = deprecated_switch_semicolon,
        rule = DeprecatedSwitchSemicolonRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1;
                    // code for case 1
                    break;
                case 2;
                    // code for case 2
                    break;
                default;
                    // default case
                    break;
            }
        "}
    }

    test_lint_success! {
        name = switch_colon_not_flagged,
        rule = DeprecatedSwitchSemicolonRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    // code for case 1
                    break;
                case 2:
                    // code for case 2
                    break;
                default:
                    // default case
                    break;
            }
        "}
    }
}
