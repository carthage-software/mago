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
pub struct NamedArgumentNameRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: NamedArgumentNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NamedArgumentNameConfig {
    pub level: Level,
}

impl Default for NamedArgumentNameConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NamedArgumentNameConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NamedArgumentNameRule {
    type Config = NamedArgumentNameConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Named Argument Name",
            code: "twig-named-argument-name",
            description: indoc! {r"
                Enforces snake_case for named arguments.

                Enable this only when the functions, filters, and macros you call follow the Twig coding standard.
                Non-ASCII names are left alone.
            "},
            good_example: indoc! {r"
                {{ input(class_name: 'input') }}
            "},
            bad_example: indoc! {r"
                {{ input(className: 'input') }}
            "},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::NamedArgument];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::NamedArgument(argument) = node else {
            return;
        };

        if !is_not_snake_case(argument.name.value) {
            return;
        }

        let name = BytesDisplay(argument.name.value);
        let snake = snake_case(argument.name.value);
        ctx.collector.report(
            Issue::new(self.cfg.level, format!("The argument name `{name}` is not in snake_case."))
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(argument.name.span).with_message("Not snake_case"))
                .with_note("The Twig coding standard spells variable, macro, and argument names in snake_case.")
                .with_note(format!(
                    "Twig's own functions, filters, and tests ignore underscores and case in argument names, so `{snake}` works there unchanged.",
                ))
                .with_help(format!(
                    "Pass the argument as `{snake}`. For a macro or method, rename its parameter first.",
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
        rule = NamedArgumentNameRule,
        code = r"{{ f(class_name: 'input') }}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = NamedArgumentNameRule,
        code = r"{{ f(className) }}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = NamedArgumentNameRule,
        count = 1,
        code = r"{{ f(className: 'input') }}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = NamedArgumentNameRule,
        count = 1,
        code = r"{{ f(className = 'input') }}",
    }
}
