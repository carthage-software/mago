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
pub struct NoRawRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: NoRawConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoRawConfig {
    pub level: Level,
}

impl Default for NoRawConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoRawConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRawRule {
    type Config = NoRawConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Raw Filter",
            code: "twig-no-raw",
            description: indoc! {r"
                Disallows the `raw` filter, including in `apply` tags. Raw output can be deliberate, so this rule
                is off by default.

                No fix is offered, because removing the filter can change escaping or token boundaries.
            "},
            good_example: indoc! {r"
                {{ user.name }}
            "},
            bad_example: indoc! {r"
                {{ user.name|raw }}
            "},
            category: Category::Security,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Filter, NodeKind::FilterApplication];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let (name, output) = match node {
            Node::Filter(filter) => (filter.name, "this value"),
            Node::FilterApplication(filter) => (filter.name, "the body of this `apply` tag"),
            _ => return,
        };

        if name.value != b"raw" {
            return;
        }

        ctx.collector.report(
            Issue::new(self.cfg.level, format!("The `raw` filter turns off escaping for {output}."))
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(name.span).with_message("Printed without escaping"))
                .with_note(
                    "Twig escapes printed output for the active strategy, usually HTML, so that user input cannot inject markup or scripts (XSS). `raw` skips that protection.",
                )
                .with_note(
                    "Only output that is already safe, such as markup the application built or sanitized itself, should be marked raw.",
                )
                .with_help(
                    "Remove the `raw` filter and let Twig escape the output. If the output is trusted markup, keep the filter and explain why in a `{# @mago-expect lint:twig-no-raw #}` comment above it.",
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
        rule = NoRawRule,
        code = r"{{ value }}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = NoRawRule,
        code = r"{{ value|escape }}",
    }

    test_lint_success! {
        name = valid_template_3,
        rule = NoRawRule,
        code = r"{{ raw(value) }}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = NoRawRule,
        count = 1,
        code = r"{{ value|raw }}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = NoRawRule,
        count = 1,
        code = r"{% apply raw %}{{ value }}{% endapply %}",
    }
}
