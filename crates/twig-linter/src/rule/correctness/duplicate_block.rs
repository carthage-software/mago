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

#[derive(Debug, Clone)]
pub struct DuplicateBlockRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: DuplicateBlockConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct DuplicateBlockConfig {
    pub level: Level,
}

impl Default for DuplicateBlockConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for DuplicateBlockConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for DuplicateBlockRule {
    type Config = DuplicateBlockConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Duplicate Block",
            code: "twig-duplicate-block",
            description: indoc! {r"
                Reports block names repeated in one template. Twig rejects such templates.

                An `embed` body has its own blocks, so it may reuse a name from the enclosing template.
            "},
            good_example: indoc! {r"
                {% block title %}Title{% endblock %}
            "},
            bad_example: indoc! {r"
                {% block title %}A{% endblock %}
                {% block title %}B{% endblock %}
            "},
            category: Category::Correctness,
            requirements: RuleRequirements::None,
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
        let mut blocks = HashMap::default();
        for node in declarations(node) {
            let Node::Block(block) = node else {
                continue;
            };

            if let Some(previous) = blocks.insert(block.name.value, block.name.span) {
                ctx.collector.report(
                    duplicate_name_issue(
                        self.cfg.level,
                        self.meta.code,
                        block.name.span,
                        previous,
                        "block",
                        "defined",
                        block.name.value,
                    )
                    .with_note("Twig rejects a template that defines the same block twice.")
                    .with_help("Rename one of the blocks, or merge their content into one block."),
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
        rule = DuplicateBlockRule,
        code = r"{% block a %}one{% endblock %}{% block b %}two{% endblock %}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = DuplicateBlockRule,
        code = r"{% block a %}one{% endblock %}{% embed 'base' %}{% block a %}two{% endblock %}{% endembed %}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = DuplicateBlockRule,
        count = 1,
        code = r"{% block a %}one{% endblock %}{% block a %}two{% endblock %}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = DuplicateBlockRule,
        count = 1,
        code = r"{% if flag %}{% block a %}one{% endblock %}{% else %}{% block a %}two{% endblock %}{% endif %}",
    }
}
