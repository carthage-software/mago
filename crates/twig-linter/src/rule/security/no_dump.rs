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
use crate::rule::utils::builtin_function;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoDumpRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: NoDumpConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoDumpConfig {
    pub level: Level,
}

impl Default for NoDumpConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoDumpConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoDumpRule {
    type Config = NoDumpConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Debug Output",
            code: "twig-no-dump",
            description: indoc! {r"
                Reports calls to the built-in `dump()` function. Debug output can expose the template context.

                A macro imported as `dump` is left alone. No fix is offered: the calls and their arguments stay
                in place.
            "},
            good_example: indoc! {r"
                {{ user.name }}
            "},
            bad_example: indoc! {r"
                {{ dump(user) }}
            "},
            category: Category::Security,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Call];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Call(call) = node else {
            return;
        };

        let Some(name) = builtin_function(ctx, call) else {
            return;
        };

        if name.name != b"dump" {
            return;
        }

        ctx.collector.report(
            Issue::new(self.cfg.level, "Remove this `dump()` call before publishing the template.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(name.span).with_message("Debug output"))
                .with_note(
                    "`dump()` prints the values you pass to it, or the whole template context when called without arguments, which can expose data the page should not show.",
                )
                .with_help(
                    "Delete the `dump()` call. While developing, inspect values with a debugger or a profiler instead of printing them in the template.",
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
        rule = NoDumpRule,
        code = r"{{ user.name }}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = NoDumpRule,
        code = r"{{ object.dump() }}",
    }

    test_lint_success! {
        name = valid_template_3,
        rule = NoDumpRule,
        code = r"{{ dump }}",
    }

    test_lint_success! {
        name = valid_template_4,
        rule = NoDumpRule,
        code = r"{% from 'macros' import x as dump %}{{ dump() }}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = NoDumpRule,
        count = 1,
        code = r"{{ dump(user) }}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = NoDumpRule,
        count = 1,
        code = r"{% do dump(user) %}",
    }

    test_lint_success! {
        name = imported_dump_macro_is_not_reported,
        rule = NoDumpRule,
        code = r"{% from 'macros' import greet as dump %}{{ dump('X') }}",
    }

    test_lint_success! {
        name = import_in_block_covers_the_block,
        rule = NoDumpRule,
        code = r"{% block body %}{% from 'macros' import greet as dump %}{{ dump('X') }}{% endblock %}",
    }

    test_lint_success! {
        name = template_import_covers_macros,
        rule = NoDumpRule,
        code = r"{% from 'macros' import greet as dump %}{% macro call() %}{{ dump('X') }}{% endmacro %}",
    }

    test_lint_failure! {
        name = block_import_does_not_leak_out,
        rule = NoDumpRule,
        count = 1,
        code = r"{% block body %}{% from 'macros' import greet as dump %}{{ dump('X') }}{% endblock %}{{ dump('X') }}",
    }

    test_lint_success! {
        name = embed_arguments_use_the_outer_imports,
        rule = NoDumpRule,
        code = r"{% from 'macros' import greet as dump %}{% embed 'card.twig' with {value: dump('X')} %}{% endembed %}",
    }

    test_lint_failure! {
        name = embed_body_does_not_see_outer_imports,
        rule = NoDumpRule,
        count = 1,
        code = r"{% from 'macros' import greet as dump %}{% embed 'card.twig' %}{% block body %}{{ dump('X') }}{% endblock %}{% endembed %}",
    }
}
