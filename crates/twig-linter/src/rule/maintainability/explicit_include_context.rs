use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
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
pub struct ExplicitIncludeContextRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: ExplicitIncludeContextConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct ExplicitIncludeContextConfig {
    pub level: Level,
}

impl Default for ExplicitIncludeContextConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for ExplicitIncludeContextConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ExplicitIncludeContextRule {
    type Config = ExplicitIncludeContextConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Explicit Include Context",
            code: "twig-explicit-include-context",
            description: indoc! {r"
                Requires `only` on `include` and `embed` tags, and a literal `with_context: false` in calls to
                the built-in `include()` function, so templates do not share the whole context by accident.

                A macro imported as `include` is left alone. No fix is offered, because it would change the
                variables the included template can see.
            "},
            good_example: indoc! {r"
                {% include 'card.twig' with {title: title} only %}
            "},
            bad_example: indoc! {r"
                {% include 'card.twig' %}
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
        let issue = match node {
            Node::Include(include) if include.only_keyword.is_none() => {
                Issue::new(self.cfg.level, "This `include` tag passes the whole template context.")
                    .with_annotation(Annotation::primary(include.keyword.span).with_message("Shares every variable"))
                    .with_help(
                        "Add `only` to the end of the tag, and pass the variables the template needs with `with {name: value}`.",
                    )
            }
            Node::Embed(embed) if embed.only_keyword.is_none() => {
                Issue::new(self.cfg.level, "This `embed` tag passes the whole template context.")
                    .with_annotation(Annotation::primary(embed.keyword.span).with_message("Shares every variable"))
                    .with_help(
                        "Add `only` to the end of the tag, and pass the variables the template needs with `with {name: value}`.",
                    )
            }
            Node::Call(call) => {
                let Some(name) = builtin_function(ctx, call).filter(|name| name.name == b"include") else {
                    return;
                };

                if matches!(include_arguments(call).with_context, Some(Expression::Bool(value)) if !value.value) {
                    return;
                }

                Issue::new(self.cfg.level, "This `include()` call passes the whole template context.")
                    .with_annotation(Annotation::primary(name.span).with_message("Shares every variable"))
                    .with_help(
                        "Add `with_context: false` to the call, and pass the variables the template needs as its second argument.",
                    )
            }
            _ => return,
        };

        ctx.collector.report(
            issue
                .with_code(self.meta.code)
                .with_note(
                    "The included template can read every variable of this one, so it can come to depend on values it never asks for.",
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
        rule = ExplicitIncludeContextRule,
        code = r"{% include 'card' with {name: name} only %}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = ExplicitIncludeContextRule,
        code = r"{% embed 'base' only %}{% endembed %}",
    }

    test_lint_success! {
        name = valid_template_3,
        rule = ExplicitIncludeContextRule,
        code = r"{{ include('card', with_context: false) }}",
    }

    test_lint_success! {
        name = valid_template_4,
        rule = ExplicitIncludeContextRule,
        code = r"{{ include('card', {}, false) }}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = ExplicitIncludeContextRule,
        count = 1,
        code = r"{% include 'card' %}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = ExplicitIncludeContextRule,
        count = 1,
        code = r"{% embed 'base' %}{% endembed %}",
    }

    test_lint_failure! {
        name = invalid_template_3,
        rule = ExplicitIncludeContextRule,
        count = 1,
        code = r"{{ include('card') }}",
    }

    test_lint_failure! {
        name = invalid_template_4,
        rule = ExplicitIncludeContextRule,
        count = 1,
        code = r"{{ include('card', with_context: flag) }}",
    }

    test_lint_success! {
        name = imported_include_macro_is_not_reported,
        rule = ExplicitIncludeContextRule,
        code = r"{% from 'macros' import greet as include %}{{ include('X') }}",
    }
}
