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
use mago_syntax::ast::UnaryPrefix;
use mago_syntax::ast::UnaryPrefixOperator;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct DeprecatedCastRule {
    meta: &'static RuleMeta,
    cfg: DeprecatedCastConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct DeprecatedCastConfig {
    pub level: Level,
}

impl Default for DeprecatedCastConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for DeprecatedCastConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for DeprecatedCastRule {
    type Config = DeprecatedCastConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Deprecated Cast",
            code: "deprecated-cast",
            description: indoc! {"
                Detect the usage of deprecated type casts in PHP code.

                In PHP 8.5, the following type casts have been deprecated:

                - `(integer)`: The integer cast has been deprecated in favor of `(int)`.
                - `(boolean)`: The boolean cast has been deprecated in favor of `(bool)`.
                - `(double)`: The double cast has been deprecated in favor of `(float)`.
                - `(binary)`: The binary cast has been deprecated in favor of `(string)`.
            "},
            good_example: indoc! {r"
                <?php

                (int) $value;
            "},
            bad_example: indoc! {r"
                <?php

                (integer) $value;
            "},
            category: Category::Deprecation,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP85)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::UnaryPrefix];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::UnaryPrefix(UnaryPrefix { operator, .. }) = node else {
            return;
        };

        let replacement = match operator {
            UnaryPrefixOperator::BooleanCast(_, _) => "(bool)",
            UnaryPrefixOperator::DoubleCast(_, _) => "(float)",
            UnaryPrefixOperator::IntegerCast(_, _) => "(int)",
            UnaryPrefixOperator::BinaryCast(_, _) => "(string)",
            _ => return,
        };

        let issue = Issue::new(self.cfg.level(), format!("Usage of deprecated cast `{operator}`."))
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(operator.span()).with_message(format!("Deprecated cast `{operator}` used here.")),
            )
            .with_note(format!("The `{operator}` cast has been deprecated since PHP 8.5."))
            .with_help(format!("Replace `{operator}` with `{replacement}` to resolve this issue."))
            .with_link("https://wiki.php.net/rfc/deprecations_php_8_5");

        ctx.collector.propose(issue, |edits| {
            edits.push(TextEdit::replace(operator.span(), replacement));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = double_deprecated,
        rule = DeprecatedCastRule,
        code = indoc! {r"
            <?php

            (double) $value;
        "}
    }

    test_lint_failure! {
        name = integer_deprecated,
        rule = DeprecatedCastRule,
        code = indoc! {r"
            <?php

            (integer) $value;
        "}
    }

    test_lint_failure! {
        name = boolean_deprecated,
        rule = DeprecatedCastRule,
        code = indoc! {r"
            <?php

            (boolean) $value;
        "}
    }

    test_lint_failure! {
        name = binary_deprecated,
        rule = DeprecatedCastRule,
        code = indoc! {r"
            <?php

            (binary) $value;
        "}
    }

    test_lint_success! {
        name = int_cast_not_flagged,
        rule = DeprecatedCastRule,
        code = indoc! {r"
            <?php

            (int) $value;
        "}
    }

    test_lint_success! {
        name = bool_cast_not_flagged,
        rule = DeprecatedCastRule,
        code = indoc! {r"
            <?php

            (bool) $value;
        "}
    }

    test_lint_success! {
        name = float_cast_not_flagged,
        rule = DeprecatedCastRule,
        code = indoc! {r"
            <?php

            (float) $value;
        "}
    }

    test_lint_success! {
        name = string_cast_not_flagged,
        rule = DeprecatedCastRule,
        code = indoc! {r"
            <?php

            (string) $value;
        "}
    }
}
