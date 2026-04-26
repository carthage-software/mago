use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Conditional;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::UnaryPrefix;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoNegatedTernaryRule {
    meta: &'static RuleMeta,
    cfg: NoNegatedTernaryConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoNegatedTernaryConfig {
    pub level: Level,
}

impl Default for NoNegatedTernaryConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoNegatedTernaryConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoNegatedTernaryRule {
    type Config = NoNegatedTernaryConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Negated Ternary",
            code: "no-negated-ternary",
            description: indoc! {"
                Flags ternary expressions whose condition is a logical negation
                (`!$foo ? a : b`).

                A negated condition adds a layer of indirection the reader has to
                undo to follow the branches. Removing the negation and swapping
                the `then` and `else` branches produces an equivalent expression
                that reads more directly.
            "},
            good_example: indoc! {r#"
                <?php

                $x = $foo ? 0 : 1;
            "#},
            bad_example: indoc! {r#"
                <?php

                $x = !$foo ? 1 : 0;
            "#},
            category: Category::Clarity,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Conditional];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Conditional(Conditional { condition, then: Some(_), .. }) = node else {
            return;
        };

        let Expression::UnaryPrefix(UnaryPrefix { operator, .. }) = condition else {
            return;
        };

        if !operator.is_not() {
            return;
        }

        ctx.collector.report(
            Issue::new(self.cfg.level(), "Negated ternary condition.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(condition.span()).with_message("This condition is negated"))
                .with_note(
                    "Reading a ternary with a negated condition forces the reader to mentally invert the branches.",
                )
                .with_help("Remove the `!` and swap the `then` and `else` branches."),
        );
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoNegatedTernaryRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = simple_negated_ternary,
        rule = NoNegatedTernaryRule,
        code = indoc! {r#"
            <?php

            $x = !$foo ? 1 : 0;
        "#}
    }

    test_lint_failure! {
        name = double_negation_still_fires,
        rule = NoNegatedTernaryRule,
        code = indoc! {r#"
            <?php

            $x = !!$foo ? 1 : 0;
        "#}
    }

    test_lint_failure! {
        name = negated_method_call,
        rule = NoNegatedTernaryRule,
        code = indoc! {r#"
            <?php

            $x = !$obj->isReady() ? 'wait' : 'go';
        "#}
    }

    test_lint_success! {
        name = positive_condition_is_kept,
        rule = NoNegatedTernaryRule,
        code = indoc! {r#"
            <?php

            $x = $foo ? 0 : 1;
        "#}
    }

    test_lint_success! {
        name = shorthand_ternary_with_negation_is_ignored,
        rule = NoNegatedTernaryRule,
        code = indoc! {r#"
            <?php

            $x = !$foo ?: 0;
        "#}
    }

    test_lint_success! {
        name = comparison_condition_is_kept,
        rule = NoNegatedTernaryRule,
        code = indoc! {r#"
            <?php

            $x = $foo === null ? 'a' : 'b';
        "#}
    }

    test_lint_success! {
        name = bitwise_not_condition_is_ignored,
        rule = NoNegatedTernaryRule,
        code = indoc! {r#"
            <?php

            $x = ~$foo ? 1 : 0;
        "#}
    }
}
