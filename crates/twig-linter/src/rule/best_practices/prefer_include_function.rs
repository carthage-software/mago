use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_bytes::BytesDisplay;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_twig_syntax::cst::Node;
use mago_twig_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::source_text;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferIncludeFunctionRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: PreferIncludeFunctionConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct PreferIncludeFunctionConfig {
    pub level: Level,
}

impl Default for PreferIncludeFunctionConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for PreferIncludeFunctionConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferIncludeFunctionRule {
    type Config = PreferIncludeFunctionConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Include Function",
            code: "twig-prefer-include-function",
            description: indoc! {r"
                Prefers the `include()` function to the `include` tag.

                No fix is offered: a rewrite must account for escaping, sandboxing, scope, and call order.
            "},
            good_example: indoc! {r"
                {{ include('card.twig') }}
            "},
            bad_example: indoc! {r"
                {% include 'card.twig' %}
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Include];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Include(include) = node else {
            return;
        };

        let template = BytesDisplay(source_text(ctx, include.template.span()));
        ctx.collector.report(
            Issue::new(self.cfg.level, "Use the `include()` function instead of the `include` tag.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(include.keyword.span).with_message("`include` tag"))
                .with_note(
                    "The Twig documentation recommends the `include()` function: it returns the rendered template, so you can pass it through filters, and it takes named options such as `with_context` and `ignore_missing`.",
                )
                .with_help(format!(
                    "Rewrite the tag as `{{{{ include({template}) }}}}`. Pass any `with` variables as the second argument, turn `only` into `with_context: false`, and `ignore missing` into `ignore_missing: true`.",
                )),
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
        rule = PreferIncludeFunctionRule,
        code = r"{{ include('card') }}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = PreferIncludeFunctionRule,
        code = r"{% embed 'card' %}{% endembed %}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = PreferIncludeFunctionRule,
        count = 1,
        code = r"{% include 'card' %}",
    }
}
