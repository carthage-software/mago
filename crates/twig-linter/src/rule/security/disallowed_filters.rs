use indoc::indoc;
use mago_allocator::Arena;
use mago_bytes::BytesDisplay;
use schemars::JsonSchema;

use foldhash::HashSet;

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
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct DisallowedFiltersRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: DisallowedFiltersConfig,
    names: HashSet<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct DisallowedFiltersConfig {
    pub level: Level,
    /// The names to disallow.
    pub names: Vec<String>,
}

impl Default for DisallowedFiltersConfig {
    fn default() -> Self {
        Self { level: Level::Warning, names: Vec::new() }
    }
}

impl Config for DisallowedFiltersConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for DisallowedFiltersRule {
    type Config = DisallowedFiltersConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Disallowed Filters",
            code: "twig-disallowed-filters",
            description: indoc! {r"
                Reports the filters listed in `names`, including in `apply` tags.

                Names match exactly. An empty list reports nothing.
            "},
            good_example: indoc! {r"
                {{ title }}
            "},
            bad_example: indoc! {r"
                {{ title|raw }}
            "},
            category: Category::Security,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Filter, NodeKind::FilterApplication];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self {
            meta: Self::meta(),
            names: settings.config.names.iter().cloned().collect(),
            cfg: settings.config.clone(),
        }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let name = match node {
            Node::Filter(filter) => filter.name,
            Node::FilterApplication(filter) => filter.name,
            _ => return,
        };

        if !std::str::from_utf8(name.value).is_ok_and(|filter| self.names.contains(filter)) {
            return;
        }

        let filter = BytesDisplay(name.value);
        ctx.collector.report(
            Issue::new(self.cfg.level, format!("The `{filter}` filter is disallowed in this project."))
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(name.span).with_message(format!("Use of `{filter}`")))
                .with_note(format!("`{filter}` is listed in `[twig.linter.rules.disallowed-filters] names`."))
                .with_help(format!(
                    "Replace the `{filter}` filter with an approach the project allows, or remove `{filter}` from the `names` list if it is now permitted.",
                )),
        );
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
        rule = DisallowedFiltersRule,
        settings = |s: &mut Settings| s.rules.disallowed_filters.config.names = vec!["unsafe_filter".into()],
        code = r"{{ value|custom_filter }}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = DisallowedFiltersRule,
        settings = |s: &mut Settings| s.rules.disallowed_filters.config.names = vec!["unsafe_filter".into()],
        code = r"{{ unsafe_filter(value) }}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = DisallowedFiltersRule,
        count = 1,
        settings = |s: &mut Settings| s.rules.disallowed_filters.config.names = vec!["unsafe_filter".into()],
        code = r"{{ value|unsafe_filter }}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = DisallowedFiltersRule,
        count = 1,
        settings = |s: &mut Settings| s.rules.disallowed_filters.config.names = vec!["unsafe_filter".into()],
        code = r"{% apply unsafe_filter %}text{% endapply %}",
    }
}
