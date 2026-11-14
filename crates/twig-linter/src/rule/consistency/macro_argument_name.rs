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
pub struct MacroArgumentNameRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: MacroArgumentNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct MacroArgumentNameConfig {
    pub level: Level,
}

impl Default for MacroArgumentNameConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for MacroArgumentNameConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for MacroArgumentNameRule {
    type Config = MacroArgumentNameConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Macro Argument Name",
            code: "twig-macro-argument-name",
            description: indoc! {r"
                Enforces snake_case for macro parameter names.

                Non-ASCII names are left alone. No fix is offered, because renaming a parameter breaks calls that
                pass it by name.
            "},
            good_example: indoc! {r"
                {% macro input(class_name) %}{% endmacro %}
            "},
            bad_example: indoc! {r"
                {% macro input(className) %}{% endmacro %}
            "},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::MacroArgument];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::MacroArgument(argument) = node else {
            return;
        };

        if !is_not_snake_case(argument.name.value) {
            return;
        }

        let name = BytesDisplay(argument.name.value);
        ctx.collector.report(
            Issue::new(self.cfg.level, format!("The macro parameter `{name}` is not in snake_case."))
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(argument.name.span).with_message("Not snake_case"))
                .with_note("The Twig coding standard spells variable, macro, and argument names in snake_case.")
                .with_help(format!(
                    "Rename the parameter to `{}` in the macro body, and in every call that passes it by name.",
                    snake_case(argument.name.value),
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
        rule = MacroArgumentNameRule,
        code = r"{% macro input(class_name, ...rest) %}{% endmacro %}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = MacroArgumentNameRule,
        count = 2,
        code = r"{% macro input(className, ...extraArgs) %}{% endmacro %}",
    }
}
