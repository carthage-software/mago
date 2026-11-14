use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_twig_syntax::cst::Expression;
use mago_twig_syntax::cst::Node;
use mago_twig_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::builtin_function;
use crate::rule::utils::include_arguments;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoIncludeIgnoreMissingRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: NoIncludeIgnoreMissingConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoIncludeIgnoreMissingConfig {
    pub level: Level,
}

impl Default for NoIncludeIgnoreMissingConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoIncludeIgnoreMissingConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoIncludeIgnoreMissingRule {
    type Config = NoIncludeIgnoreMissingConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Ignored Missing Include",
            code: "twig-no-include-ignore-missing",
            description: indoc! {r"
                Disallows `ignore missing` on `include` and `embed` tags, and a literal `ignore_missing: true`
                in calls to the built-in `include()` function.

                Dynamic values are left alone.
            "},
            good_example: indoc! {r"
                {% include 'card.twig' %}
            "},
            bad_example: indoc! {r"
                {% include 'card.twig' ignore missing %}
            "},
            category: Category::Maintainability,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Include, NodeKind::Embed, NodeKind::Call];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let (span, issue) = match node {
            Node::Include(include) => match &include.ignore_missing {
                Some(missing) => (
                    missing.span(),
                    Issue::new(self.cfg.level, "`ignore missing` hides a missing template.")
                        .with_help("Remove `ignore missing`, so that Twig reports a missing template instead of rendering nothing."),
                ),
                None => return,
            },
            Node::Embed(embed) => match &embed.ignore_missing {
                Some(missing) => (
                    missing.span(),
                    Issue::new(self.cfg.level, "`ignore missing` hides a missing template.")
                        .with_help("Remove `ignore missing`, so that Twig reports a missing template instead of rendering nothing."),
                ),
                None => return,
            },
            Node::Call(call) => {
                let Some(name) = builtin_function(ctx, call).filter(|name| name.name == b"include") else {
                    return;
                };

                let Some(value) = include_arguments(call).ignore_missing else {
                    return;
                };

                if !matches!(value, Expression::Bool(value) if value.value) {
                    return;
                }

                (
                    name.span,
                    Issue::new(self.cfg.level, "`ignore_missing: true` hides a missing template.")
                        .with_annotation(Annotation::secondary(value.span()).with_message("Set to `true` here"))
                        .with_help("Remove `ignore_missing: true`, so that Twig reports a missing template instead of rendering nothing."),
                )
            }
            _ => return,
        };

        ctx.collector.report(
            issue
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(span).with_message("A missing template renders nothing"))
                .with_note("If the template is renamed or deleted, the page renders without it and nothing reports the mistake.")
                .with_note(
                    "When the template is truly optional, keep the option and explain why in a `{# @mago-expect lint:twig-no-include-ignore-missing #}` comment.",
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
        rule = NoIncludeIgnoreMissingRule,
        code = r"{% include 'card' %}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = NoIncludeIgnoreMissingRule,
        code = r"{{ include('card', ignore_missing: false) }}",
    }

    test_lint_success! {
        name = valid_template_3,
        rule = NoIncludeIgnoreMissingRule,
        code = r"{{ include('card', ignore_missing: flag) }}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = NoIncludeIgnoreMissingRule,
        count = 1,
        code = r"{% include 'card' ignore missing %}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = NoIncludeIgnoreMissingRule,
        count = 1,
        code = r"{% embed 'card' ignore missing %}{% endembed %}",
    }

    test_lint_failure! {
        name = invalid_template_3,
        rule = NoIncludeIgnoreMissingRule,
        count = 1,
        code = r"{{ include('card', ignore_missing: true) }}",
    }

    test_lint_failure! {
        name = invalid_template_4,
        rule = NoIncludeIgnoreMissingRule,
        count = 1,
        code = r"{{ include('card', {}, true, true) }}",
    }
}
