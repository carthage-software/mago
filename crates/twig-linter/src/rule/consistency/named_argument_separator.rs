use indoc::indoc;
use mago_allocator::Arena;
use mago_bytes::BytesDisplay;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_text_edit::TextEdit;
use mago_twig_syntax::cst::NamedArgumentSeparator;
use mago_twig_syntax::cst::Node;
use mago_twig_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;
use crate::version::TwigVersion;

#[derive(Debug, Clone)]
pub struct NamedArgumentSeparatorRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: NamedArgumentSeparatorConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NamedArgumentSeparatorConfig {
    pub level: Level,
}

impl Default for NamedArgumentSeparatorConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NamedArgumentSeparatorConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NamedArgumentSeparatorRule {
    type Config = NamedArgumentSeparatorConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Named Argument Separator",
            code: "twig-named-argument-separator",
            description: indoc! {r"
                Requires `:` between a named argument and its value, as the Twig coding standard recommends,
                instead of `=`.

                Macro parameter defaults keep `=`.
            "},
            good_example: indoc! {r"
                {{ range(low: 1, high: 3) }}
            "},
            bad_example: indoc! {r"
                {{ range(low = 1, high = 3) }}
            "},
            category: Category::Consistency,
            requirements: RuleRequirements::TwigVersion(TwigVersion::new(12)),
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

        let NamedArgumentSeparator::Equal(separator) = argument.separator else {
            return;
        };

        let name = BytesDisplay(argument.name.value);
        ctx.collector.propose(
            Issue::new(self.cfg.level, format!("Use `:` instead of `=` after the argument name `{name}`."))
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(separator).with_message("`=` separator"))
                .with_note(
                    "Twig 3.12 added `name: value` for named arguments, and the Twig coding standard prefers it to `name = value`. Macro parameter defaults still use `=`.",
                )
                .with_help(format!("Write `{name}: value` instead of `{name} = value`.")),
            |edits| edits.push(TextEdit::replace(separator.start_offset()..separator.end_offset(), ":")),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_success! {
        name = valid_template_1,
        rule = NamedArgumentSeparatorRule,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{{ f(a: 1) }}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = NamedArgumentSeparatorRule,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{% macro f(a = 1) %}{% endmacro %}",
    }

    test_lint_success! {
        name = valid_template_3,
        rule = NamedArgumentSeparatorRule,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{{ f({a: 1}) }}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = NamedArgumentSeparatorRule,
        count = 1,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{{ f(a = 1) }}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = NamedArgumentSeparatorRule,
        count = 1,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{{ x|custom(a=1) }}",
    }

    test_lint_failure! {
        name = invalid_template_3,
        rule = NamedArgumentSeparatorRule,
        count = 1,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{{ x is custom(a = 1) }}",
    }

    test_lint_fix! {
        name = equal_signs_become_colons,
        rule = NamedArgumentSeparatorRule,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{{ range(low=1, high=3) }}",
        fixed = r"{{ range(low:1, high:3) }}",
    }
}
