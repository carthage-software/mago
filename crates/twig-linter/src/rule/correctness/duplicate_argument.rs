use std::borrow::Cow;

use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use foldhash::HashMap;

use mago_reporting::Level;
use mago_twig_syntax::cst::Argument;
use mago_twig_syntax::cst::Expression;
use mago_twig_syntax::cst::Node;
use mago_twig_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::duplicate_name_issue;
use crate::rule::utils::normalize_argument_name;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct DuplicateArgumentRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: DuplicateArgumentConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct DuplicateArgumentConfig {
    pub level: Level,
}

impl Default for DuplicateArgumentConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for DuplicateArgumentConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for DuplicateArgumentRule {
    type Config = DuplicateArgumentConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Duplicate Named Argument",
            code: "twig-duplicate-argument",
            description: indoc! {r"
                Reports named arguments that repeat a name in one call, filter, or test.

                Twig ignores underscores and case in the argument names of its own functions, filters, and
                tests, so `first_name` and `firstName` name the same argument there. Calls to macros and
                methods compare names exactly.
            "},
            good_example: indoc! {r"
                {{ range(low: 1, high: 3) }}
            "},
            bad_example: indoc! {r"
                {{ range(low: 1, low: 3) }}
            "},
            category: Category::Correctness,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::ArgumentList];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::ArgumentList(arguments) = node else {
            return;
        };

        if arguments.arguments.len() < 2 {
            return;
        }

        let normalized = match ctx.get_parent() {
            Some(Node::Call(call)) => matches!(call.callee, Expression::Name(name) if !ctx.is_imported(name.name)),
            Some(Node::MethodCall(_)) => false,
            _ => true,
        };

        let mut names = HashMap::default();
        for argument in arguments.arguments.iter() {
            let Argument::Named(argument) = argument else {
                continue;
            };

            let key = if normalized {
                normalize_argument_name(argument.name.value)
            } else {
                Cow::Borrowed(argument.name.value)
            };

            let Some(previous) = names.insert(key, argument.name.span) else {
                continue;
            };

            let mut issue = duplicate_name_issue(
                self.cfg.level,
                self.meta.code,
                argument.name.span,
                previous,
                "argument",
                "passed",
                argument.name.value,
            )
            .with_note("A call cannot pass the same argument twice: Twig rejects it.");

            if normalized {
                issue = issue.with_note(
                    "Twig ignores underscores and case in the argument names of its own functions, filters, and tests, so `first_name` and `firstName` are the same argument there.",
                );
            }

            ctx.collector.report(
                issue.with_help(
                    "Remove one of the two arguments, or rename it if you meant to pass a different argument.",
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = valid_template_1,
        rule = DuplicateArgumentRule,
        code = r"{{ f(a: 1, b: 2) }}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = DuplicateArgumentRule,
        code = r"{{ f(a: 1) + f(a: 2) }}",
    }

    test_lint_success! {
        name = valid_template_3,
        rule = DuplicateArgumentRule,
        code = r"{% macro f(a = 1, b = 2) %}{% endmacro %}",
    }

    test_lint_success! {
        name = valid_template_4,
        rule = DuplicateArgumentRule,
        code = r"{% macro f(foo_bar, foobar) %}{% endmacro %}{{ _self.f(foo_bar: 1, foobar: 2) }}",
    }

    test_lint_success! {
        name = valid_template_5,
        rule = DuplicateArgumentRule,
        code = r"{% from 'macros' import f %}{{ f(foo_bar: 1, foobar: 2) }}",
    }

    test_lint_success! {
        name = valid_template_6,
        rule = DuplicateArgumentRule,
        code = r"{{ object.f(foo_bar: 1, foobar: 2) }}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = DuplicateArgumentRule,
        count = 1,
        code = r"{{ f(a: 1, a: 2) }}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = DuplicateArgumentRule,
        count = 1,
        code = r"{{ x|custom(a = 1, a: 2) }}",
    }

    test_lint_failure! {
        name = invalid_template_3,
        rule = DuplicateArgumentRule,
        count = 1,
        code = r"{{ x is custom(a: 1, a: 2) }}",
    }

    test_lint_failure! {
        name = invalid_template_4,
        rule = DuplicateArgumentRule,
        count = 2,
        code = r"{{ f(firstName: 1, first_name: 2, FIRST_NAME: 3) }}",
    }

    test_lint_failure! {
        name = invalid_template_5,
        rule = DuplicateArgumentRule,
        count = 1,
        code = r"{{ x|custom(firstName: 1, first_name: 2) }}",
    }

    test_lint_failure! {
        name = invalid_template_6,
        rule = DuplicateArgumentRule,
        count = 1,
        code = r"{{ x is custom(firstName: 1, first_name: 2) }}",
    }

    test_lint_failure! {
        name = invalid_template_7,
        rule = DuplicateArgumentRule,
        count = 1,
        code = r"{% from 'macros' import f as custom %}{{ x is custom(firstName: 1, first_name: 2) }}",
    }

    test_lint_failure! {
        name = invalid_template_8,
        rule = DuplicateArgumentRule,
        count = 1,
        code = r"{{ f(foo_bar: 1, foobar: 2) }}",
    }

    test_lint_failure! {
        name = invalid_template_9,
        rule = DuplicateArgumentRule,
        count = 1,
        code = r"{{ x|custom(foo_bar: 1, foobar: 2) }}",
    }

    test_lint_failure! {
        name = invalid_template_10,
        rule = DuplicateArgumentRule,
        count = 1,
        code = r"{{ x is custom(foo_bar: 1, foobar: 2) }}",
    }

    test_lint_failure! {
        name = apply_filter_arguments_compare_normalized_names,
        rule = DuplicateArgumentRule,
        count = 1,
        code = r"{% apply custom(first_name: 1, firstName: 2) %}text{% endapply %}",
    }
}
