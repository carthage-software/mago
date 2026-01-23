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
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoIssetRule {
    meta: &'static RuleMeta,
    cfg: NoIssetConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoIssetConfig {
    pub level: Level,
}

impl Default for NoIssetConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoIssetConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoIssetRule {
    type Config = NoIssetConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Isset",
            code: "no-isset",
            description: indoc! {"
                Detects the use of the `isset()` construct.

                The `isset()` language construct checks whether a variable is set and is not null.
                However, it can lead to ambiguous code because it conflates two distinct checks:
                variable existence and null comparison. Using explicit null checks or the null
                coalescing operator (`??`) is often clearer and more maintainable.
            "},
            good_example: indoc! {r"
                <?php

                if ($value !== null) {
                    // ...
                }

                $result = $value ?? 'default';
            "},
            bad_example: indoc! {r"
                <?php

                if (isset($value)) {
                    // ...
                }
            "},
            category: Category::Clarity,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::IssetConstruct];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::IssetConstruct(construct) = node else {
            return;
        };

        ctx.collector.report(
            Issue::new(self.cfg.level(), "Use of the `isset` construct.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(construct.span())
                        .with_message("Ambiguous check due to `isset()` conflating existence and null comparison"),
                )
                .with_note("`isset()` returns false for both unset variables and null values.")
                .with_note("It is unclear whether you're checking for variable existence or non-null value.")
                .with_help("Use explicit null comparison (`!== null`) or the null coalescing operator (`??`)."),
        );
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoIssetRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = null_comparison_is_allowed,
        rule = NoIssetRule,
        code = indoc! {r"
            <?php

            if ($value !== null) {
                echo $value;
            }
        "}
    }

    test_lint_success! {
        name = null_coalescing_is_allowed,
        rule = NoIssetRule,
        code = indoc! {r"
            <?php

            $result = $value ?? 'default';
        "}
    }

    test_lint_success! {
        name = array_key_exists_is_allowed,
        rule = NoIssetRule,
        code = indoc! {r"
            <?php

            if (array_key_exists('key', $array)) {
                echo $array['key'];
            }
        "}
    }

    test_lint_failure! {
        name = simple_isset_fails,
        rule = NoIssetRule,
        code = indoc! {r"
            <?php

            if (isset($value)) {
                echo $value;
            }
        "}
    }

    test_lint_failure! {
        name = isset_with_array_key_fails,
        rule = NoIssetRule,
        code = indoc! {r#"
            <?php

            if (isset($array['key'])) {
                echo $array['key'];
            }
        "#}
    }

    test_lint_failure! {
        name = isset_with_multiple_args_fails,
        rule = NoIssetRule,
        code = indoc! {r"
            <?php

            if (isset($a, $b, $c)) {
                echo 'all set';
            }
        "}
    }

    test_lint_failure! {
        name = negated_isset_fails,
        rule = NoIssetRule,
        code = indoc! {r"
            <?php

            if (!isset($value)) {
                $value = 'default';
            }
        "}
    }

    test_lint_failure! {
        name = isset_in_ternary_fails,
        rule = NoIssetRule,
        code = indoc! {r"
            <?php

            $result = isset($value) ? $value : 'default';
        "}
    }
}
