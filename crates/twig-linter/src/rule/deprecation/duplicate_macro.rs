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
use crate::rule::utils::declarations;
use crate::rule::utils::duplicate_name_issue;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;
use crate::version::TwigVersion;

#[derive(Debug, Clone)]
pub struct DuplicateMacroRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: DuplicateMacroConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct DuplicateMacroConfig {
    pub level: Level,
}

impl Default for DuplicateMacroConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for DuplicateMacroConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for DuplicateMacroRule {
    type Config = DuplicateMacroConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Duplicate Macro",
            code: "twig-duplicate-macro",
            description: indoc! {r"
                Reports macro names repeated in one template. Names are case-sensitive.

                Twig 3 uses the last definition; Twig 3.29 deprecates this and Twig 4 rejects it. An `embed` body
                has its own macros.
            "},
            good_example: indoc! {r"
                {% macro title() %}Title{% endmacro %}
            "},
            bad_example: indoc! {r"
                {% macro title() %}A{% endmacro %}
                {% macro title() %}B{% endmacro %}
            "},
            category: Category::Deprecation,
            requirements: RuleRequirements::TwigVersion(TwigVersion::new(29)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Template, NodeKind::Embed];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let mut macros = HashMap::default();
        for node in declarations(node) {
            let Node::Macro(r#macro) = node else {
                continue;
            };

            if let Some(previous) = macros.insert(r#macro.name.value, r#macro.name.span) {
                ctx.collector.report(
                    duplicate_name_issue(
                        self.cfg.level,
                        self.meta.code,
                        r#macro.name.span,
                        previous,
                        "macro",
                        "defined",
                        r#macro.name.value,
                    )
                    .with_note(
                        "Twig 3 keeps the last definition and ignores the earlier one. Twig 3.29 deprecates this, and Twig 4 rejects it.",
                    )
                    .with_help("Rename one of the macros, or delete the one you no longer need."),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = valid_template_1,
        rule = DuplicateMacroRule,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{% macro a() %}one{% endmacro %}{% macro b() %}two{% endmacro %}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = DuplicateMacroRule,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{% macro row() %}one{% endmacro %}{% macro Row() %}two{% endmacro %}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = DuplicateMacroRule,
        count = 1,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{% macro a() %}one{% endmacro %}{% macro a() %}two{% endmacro %}",
    }
}
