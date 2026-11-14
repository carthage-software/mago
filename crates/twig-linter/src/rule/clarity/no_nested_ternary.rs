use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_twig_syntax::cst::Node;
use mago_twig_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoNestedTernaryRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: NoNestedTernaryConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoNestedTernaryConfig {
    pub level: Level,
}

impl Default for NoNestedTernaryConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoNestedTernaryConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoNestedTernaryRule {
    type Config = NoNestedTernaryConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Nested Ternary",
            code: "twig-no-nested-ternary",
            description: indoc! {r"
                Reports conditional expressions nested inside other conditional expressions. Split them into
                named values or branches.

                Parentheses do not hide a nested conditional. A conditional in an arrow function body is not
                nested in the conditional around the arrow function.
            "},
            good_example: indoc! {r"
                {{ flag ? yes : no }}
            "},
            bad_example: indoc! {r"
                {{ flag ? (other ? one : two) : no }}
            "},
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
        let Node::Conditional(conditional) = node else {
            return;
        };

        if !ctx.is_in_conditional() {
            return;
        }

        ctx.collector.report(
            Issue::new(self.cfg.level, "This conditional expression is nested inside another one.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(conditional.question_mark).with_message("Nested conditional"))
                .with_note("Nested conditionals are hard to read, and it is easy to misread which value belongs to which condition.")
                .with_help(
                    "Give the inner result a name with `{% set %}` before this expression, or split the logic into `{% if %}` branches.",
                ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = valid_template_1,
        rule = NoNestedTernaryRule,
        code = r"{{ flag ? yes : no }}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = NoNestedTernaryRule,
        code = r"{{ flag ? yes : no }}{{ other ? one : two }}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = NoNestedTernaryRule,
        count = 1,
        code = r"{{ flag ? (other ? one : two) : no }}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = NoNestedTernaryRule,
        count = 1,
        code = r"{{ flag ? yes : (other ? one : two) }}",
    }

    test_lint_failure! {
        name = invalid_template_3,
        rule = NoNestedTernaryRule,
        count = 1,
        code = r"{{ (other ? one : two) ? yes : no }}",
    }

    test_lint_failure! {
        name = conditional_in_an_argument_is_nested,
        rule = NoNestedTernaryRule,
        count = 1,
        code = r"{{ flag ? format(other ? one : two) : no }}",
    }

    test_lint_success! {
        name = conditional_in_an_arrow_function_is_not_nested,
        rule = NoNestedTernaryRule,
        code = r"{{ flag ? items|map(item => item ? one : two) : none }}",
    }
}
