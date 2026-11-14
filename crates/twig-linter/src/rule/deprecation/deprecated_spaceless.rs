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
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;
use crate::version::TwigVersion;

#[derive(Debug, Clone)]
pub struct DeprecatedSpacelessRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: DeprecatedSpacelessConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct DeprecatedSpacelessConfig {
    pub level: Level,
}

impl Default for DeprecatedSpacelessConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for DeprecatedSpacelessConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for DeprecatedSpacelessRule {
    type Config = DeprecatedSpacelessConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Deprecated Spaceless Filter",
            code: "twig-deprecated-spaceless",
            description: indoc! {r"
                Reports the `spaceless` filter, deprecated since Twig 3.12, including in `apply` tags.

                No fix is offered: check the rendered output before you stop stripping whitespace.
            "},
            good_example: indoc! {r"
                {{ text }}
            "},
            bad_example: indoc! {r"
                {{ text|spaceless }}
            "},
            category: Category::Deprecation,
            requirements: RuleRequirements::TwigVersion(TwigVersion::new(12)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Filter, NodeKind::FilterApplication];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
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

        if name.value != b"spaceless" {
            return;
        }

        ctx.collector.report(
            Issue::new(self.cfg.level, "The `spaceless` filter is deprecated since Twig 3.12.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(name.span).with_message("Deprecated filter"))
                .with_note(
                    "`spaceless` removes the whitespace between HTML tags, which can change how a page looks, such as the gaps between inline elements.",
                )
                .with_help(
                    "Remove the filter, then control whitespace with Twig's whitespace modifiers, such as `{%- if flag -%}` and `{{- value -}}`, or minify the HTML after rendering.",
                ),
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
        rule = DeprecatedSpacelessRule,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{{ spaceless }}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = DeprecatedSpacelessRule,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{{ x|custom }}",
    }

    test_lint_success! {
        name = valid_template_3,
        rule = DeprecatedSpacelessRule,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{{ custom.spaceless }}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = DeprecatedSpacelessRule,
        count = 1,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{{ x|spaceless }}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = DeprecatedSpacelessRule,
        count = 1,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{% apply spaceless %}<b> x </b>{% endapply %}",
    }
}
