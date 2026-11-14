use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use foldhash::HashMap;

use mago_reporting::Level;
use mago_twig_syntax::cst::Node;
use mago_twig_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::duplicate_name_issue;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct DuplicateParameterRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: DuplicateParameterConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct DuplicateParameterConfig {
    pub level: Level,
}

impl Default for DuplicateParameterConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for DuplicateParameterConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for DuplicateParameterRule {
    type Config = DuplicateParameterConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Duplicate Macro Parameter",
            code: "twig-duplicate-parameter",
            description: indoc! {r"
                Reports macro parameter names that repeat an earlier parameter, including variadic parameters.
                Twig rejects such macros.
            "},
            good_example: indoc! {r"
                {% macro input(name, value) %}{% endmacro %}
            "},
            bad_example: indoc! {r"
                {% macro input(name, name) %}{% endmacro %}
            "},
            category: Category::Correctness,
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

        let mut parameters = HashMap::default();
        for parameter in r#macro.arguments.iter() {
            if let Some(previous) = parameters.insert(parameter.name.value, parameter.name.span) {
                ctx.collector.report(
                    duplicate_name_issue(
                        self.cfg.level,
                        self.meta.code,
                        parameter.name.span,
                        previous,
                        "parameter",
                        "declared",
                        parameter.name.value,
                    )
                    .with_note("Twig rejects a macro that declares the same parameter twice.")
                    .with_help("Rename or remove one of the parameters."),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = valid_template_1,
        rule = DuplicateParameterRule,
        code = r"{% macro a(first, second = 'x', ...rest) %}{% endmacro %}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = DuplicateParameterRule,
        count = 1,
        code = r"{% macro a(first, first) %}{% endmacro %}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = DuplicateParameterRule,
        count = 1,
        code = r"{% macro a(first, ...first) %}{% endmacro %}",
    }

    test_lint_failure! {
        name = variadic_parameter_repeats_a_name,
        rule = DuplicateParameterRule,
        count = 1,
        code = r"{% macro input(values, ...values) %}{% endmacro %}",
    }
}
