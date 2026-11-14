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
use crate::version::TwigVersion;

#[derive(Debug, Clone)]
pub struct DeprecatedAttributeRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: DeprecatedAttributeConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct DeprecatedAttributeConfig {
    pub level: Level,
}

impl Default for DeprecatedAttributeConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for DeprecatedAttributeConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for DeprecatedAttributeRule {
    type Config = DeprecatedAttributeConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Deprecated Attribute Function",
            code: "twig-deprecated-attribute",
            description: indoc! {r"
                Reports calls to the built-in `attribute()` function, deprecated since Twig 3.15.

                Use dynamic attribute access, `object.(name)`, after checking its call and lookup behavior. A
                macro imported as `attribute` is left alone.
            "},
            good_example: indoc! {r"
                {{ object.(name) }}
            "},
            bad_example: indoc! {r"
                {{ attribute(object, name) }}
            "},
            category: Category::Deprecation,
            requirements: RuleRequirements::TwigVersion(TwigVersion::new(15)),
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

        if name.name != b"attribute" {
            return;
        }

        ctx.collector.report(
            Issue::new(self.cfg.level, "The `attribute()` function is deprecated since Twig 3.15.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(name.span).with_message("Deprecated function"))
                .with_note("Twig 3.15 added dynamic attribute access, `object.(name)`, which replaces `attribute(object, name)`.")
                .with_help(
                    "Rewrite `attribute(object, name)` as `object.(name)`, and `attribute(object, name, arguments)` as `object.(name)(...arguments)`.",
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
        rule = DeprecatedAttributeRule,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{{ object.(name) }}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = DeprecatedAttributeRule,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{{ object.attribute(name) }}",
    }

    test_lint_success! {
        name = valid_template_3,
        rule = DeprecatedAttributeRule,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{% from 'macros' import x as attribute %}{{ attribute(1) }}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = DeprecatedAttributeRule,
        count = 1,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{{ attribute(object, name) }}",
    }

    test_lint_success! {
        name = imported_attribute_macro_is_not_reported,
        rule = DeprecatedAttributeRule,
        settings = |s: &mut Settings| s.version = TwigVersion::new(30),
        code = r"{% from 'macros' import greet as attribute %}{{ attribute('X') }}",
    }
}
