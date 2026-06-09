use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Binary;
use mago_syntax::ast::BinaryOperator;
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
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
                Flags ternary expressions whose condition is a negated form,
                including logical negation (`!$foo ? a : b`) and the
                not-equal comparison operators (`!==`, `!=`, `<>`).

                A negated condition adds a layer of indirection the reader has to
                undo to follow the branches. Inverting the condition and swapping
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
                $y = $foo !== null ? transform($foo) : null;
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

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Conditional(Conditional { condition, then: Some(_), .. }) = node else {
            return;
        };

        let Some(NegatedCondition { annotation_span, help }) = classify_negated_condition(condition) else {
            return;
        };

        ctx.collector.report(
            Issue::new(self.cfg.level(), "Negated ternary condition.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(annotation_span).with_message("This condition is negated"))
                .with_note(
                    "Reading a ternary with a negated condition forces the reader to mentally invert the branches.",
                )
                .with_help(help),
        );
    }
}

struct NegatedCondition {
    annotation_span: Span,
    help: &'static str,
}

fn classify_negated_condition(condition: &Expression<'_>) -> Option<NegatedCondition> {
    if let Expression::UnaryPrefix(UnaryPrefix { operator, .. }) = condition
        && operator.is_not()
    {
        return Some(NegatedCondition {
            annotation_span: condition.span(),
            help: "Remove the `!` and swap the `then` and `else` branches.",
        });
    }

    if let Expression::Binary(Binary { operator, .. }) = condition {
        return match operator {
            BinaryOperator::NotIdentical(_) => Some(NegatedCondition {
                annotation_span: operator.span(),
                help: "Replace `!==` with `===` and swap the `then` and `else` branches.",
            }),
            BinaryOperator::NotEqual(_) => Some(NegatedCondition {
                annotation_span: operator.span(),
                help: "Replace `!=` with `==` and swap the `then` and `else` branches.",
            }),
            BinaryOperator::AngledNotEqual(_) => Some(NegatedCondition {
                annotation_span: operator.span(),
                help: "Replace `<>` with `==` and swap the `then` and `else` branches.",
            }),
            _ => None,
        };
    }

    None
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

    test_lint_failure! {
        name = not_identical_condition,
        rule = NoNegatedTernaryRule,
        code = indoc! {r#"
            <?php

            $x = $foo !== null ? transform($foo) : null;
        "#}
    }

    test_lint_failure! {
        name = not_equal_condition,
        rule = NoNegatedTernaryRule,
        code = indoc! {r#"
            <?php

            $x = $foo != 0 ? 'nonzero' : 'zero';
        "#}
    }

    test_lint_failure! {
        name = angled_not_equal_condition,
        rule = NoNegatedTernaryRule,
        code = indoc! {r#"
            <?php

            $x = $foo <> 0 ? 'nonzero' : 'zero';
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

    test_lint_success! {
        name = positive_identity_condition_is_kept,
        rule = NoNegatedTernaryRule,
        code = indoc! {r#"
            <?php

            $x = $foo === null ? 'null' : 'value';
        "#}
    }

    test_lint_success! {
        name = relational_condition_is_kept,
        rule = NoNegatedTernaryRule,
        code = indoc! {r#"
            <?php

            $x = $foo < 0 ? 'negative' : 'positive';
        "#}
    }

    test_lint_success! {
        name = not_equal_in_shorthand_ternary_is_ignored,
        rule = NoNegatedTernaryRule,
        code = indoc! {r#"
            <?php

            $x = $foo !== null ?: 'fallback';
        "#}
    }
}
