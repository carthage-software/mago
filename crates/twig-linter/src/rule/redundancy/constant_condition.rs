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
use crate::rule::utils::ungroup;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct ConstantConditionRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: ConstantConditionConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct ConstantConditionConfig {
    pub level: Level,
}

impl Default for ConstantConditionConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for ConstantConditionConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ConstantConditionRule {
    type Config = ConstantConditionConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Constant Condition",
            code: "twig-constant-condition",
            description: indoc! {r"
                Reports literal boolean conditions in `if` branches and conditional expressions.

                The rule does not guess the truthiness of other values. No fix is offered, because removing a
                branch removes rendered text.
            "},
            good_example: indoc! {r"
                {% if flag %}yes{% endif %}
            "},
            bad_example: indoc! {r"
                {% if true %}yes{% endif %}
            "},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::IfBranch, NodeKind::Conditional];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let (condition, is_tag) = match node {
            Node::IfBranch(branch) => (&branch.condition, true),
            Node::Conditional(conditional) => (conditional.condition, false),
            _ => return,
        };

        let Expression::Bool(literal) = ungroup(condition) else {
            return;
        };

        let help = match (is_tag, literal.value) {
            (true, true) => "Remove the condition and keep the body of this branch. Any branches after it never run.",
            (true, false) => "Remove this branch: its body never renders.",
            (false, true) => "Replace the conditional expression with the value it returns when the condition holds.",
            (false, false) => "Replace the conditional expression with the value it returns when the condition fails.",
        };

        ctx.collector.report(
            Issue::new(self.cfg.level, format!("This condition is always `{}`.", literal.value))
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(condition.span()).with_message(format!("Always `{}`", literal.value)),
                )
                .with_note("A literal condition never changes, so one path always runs and the other is dead code.")
                .with_help(help),
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
        rule = ConstantConditionRule,
        code = r"{% if flag %}yes{% endif %}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = ConstantConditionRule,
        code = r"{% if 1 %}yes{% endif %}",
    }

    test_lint_success! {
        name = valid_template_3,
        rule = ConstantConditionRule,
        code = r"{{ flag ? yes : no }}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = ConstantConditionRule,
        count = 2,
        code = r"{% if true %}yes{% elseif false %}no{% endif %}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = ConstantConditionRule,
        count = 1,
        code = r"{{ (false) ? yes : no }}",
    }
}
