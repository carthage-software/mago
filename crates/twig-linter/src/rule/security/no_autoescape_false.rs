use indoc::indoc;
use mago_allocator::Arena;
use mago_span::HasSpan;
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
use crate::rule::utils::ungroup;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoAutoescapeFalseRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: NoAutoescapeFalseConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoAutoescapeFalseConfig {
    pub level: Level,
}

impl Default for NoAutoescapeFalseConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoAutoescapeFalseConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoAutoescapeFalseRule {
    type Config = NoAutoescapeFalseConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Disabled Autoescaping",
            code: "twig-no-autoescape-false",
            description: indoc! {r"
                Disallows `autoescape false` blocks, which turn off escaping for their whole body.

                No fix is offered, because Mago cannot tell which output needs escaping.
            "},
            good_example: indoc! {r"
                {% autoescape 'html' %}{{ value }}{% endautoescape %}
            "},
            bad_example: indoc! {r"
                {% autoescape false %}{{ value }}{% endautoescape %}
            "},
            category: Category::Security,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Autoescape];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Autoescape(autoescape) = node else {
            return;
        };

        let Some(strategy) = &autoescape.strategy else {
            return;
        };

        if !matches!(ungroup(strategy), Expression::Bool(value) if !value.value) {
            return;
        }

        ctx.collector.report(
            Issue::new(self.cfg.level, "This `autoescape false` block turns off escaping for its whole body.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(autoescape.keyword.span).with_message("Escaping is off inside this block"))
                .with_annotation(Annotation::secondary(strategy.span()).with_message("`false` turns escaping off"))
                .with_note(
                    "Inside this block, Twig prints every value as it is, so a value that holds user input can inject markup or scripts (XSS).",
                )
                .with_help(
                    "Remove the `autoescape false` block, or name the strategy the output needs, such as `{% autoescape 'js' %}`. Mark the few values that are already safe with the `raw` filter instead.",
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
        rule = NoAutoescapeFalseRule,
        code = r"{% autoescape 'html' %}{{ value }}{% endautoescape %}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = NoAutoescapeFalseRule,
        code = r"{% autoescape 'false' %}{{ value }}{% endautoescape %}",
    }

    test_lint_success! {
        name = valid_template_3,
        rule = NoAutoescapeFalseRule,
        code = r"{% autoescape %}{{ value }}{% endautoescape %}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = NoAutoescapeFalseRule,
        count = 1,
        code = r"{% autoescape false %}{{ value }}{% endautoescape %}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = NoAutoescapeFalseRule,
        count = 1,
        code = r"{% autoescape (false) %}{{ value }}{% endautoescape %}",
    }
}
