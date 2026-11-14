use indoc::indoc;
use mago_allocator::Arena;
use mago_bytes::BytesDisplay;
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
use crate::rule::utils::is_not_snake_case;
use crate::rule::utils::snake_case;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct MacroNameRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: MacroNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct MacroNameConfig {
    pub level: Level,
}

impl Default for MacroNameConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for MacroNameConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for MacroNameRule {
    type Config = MacroNameConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Macro Name",
            code: "twig-macro-name",
            description: indoc! {r"
                Enforces snake_case for macro names.

                Non-ASCII names are left alone. No fix is offered, because renaming a macro breaks its callers.
            "},
            good_example: indoc! {r"
                {% macro html_input() %}{% endmacro %}
            "},
            bad_example: indoc! {r"
                {% macro htmlInput() %}{% endmacro %}
            "},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Macro];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Macro(r#macro) = node else {
            return;
        };

        if !is_not_snake_case(r#macro.name.value) {
            return;
        }

        let name = BytesDisplay(r#macro.name.value);
        ctx.collector.report(
            Issue::new(self.cfg.level, format!("The macro name `{name}` is not in snake_case."))
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(r#macro.name.span).with_message("Not snake_case"))
                .with_note("The Twig coding standard spells variable, macro, and argument names in snake_case.")
                .with_help(format!(
                    "Rename the macro to `{}`, and update every template that calls it.",
                    snake_case(r#macro.name.value),
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
        rule = MacroNameRule,
        code = r"{% macro html_input() %}{% endmacro %}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = MacroNameRule,
        code = r"{{ htmlInput() }}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = MacroNameRule,
        count = 1,
        code = r"{% macro htmlInput() %}{% endmacro %}",
    }
}
