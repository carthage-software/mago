use indoc::indoc;
use mago_allocator::Arena;
use mago_bytes::BytesDisplay;
use schemars::JsonSchema;

use foldhash::HashSet;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_twig_syntax::cst::Expression;
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
pub struct DisallowedFunctionsRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: DisallowedFunctionsConfig,
    names: HashSet<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct DisallowedFunctionsConfig {
    pub level: Level,
    /// The names to disallow.
    pub names: Vec<String>,
}

impl Default for DisallowedFunctionsConfig {
    fn default() -> Self {
        Self { level: Level::Warning, names: Vec::new() }
    }
}

impl Config for DisallowedFunctionsConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for DisallowedFunctionsRule {
    type Config = DisallowedFunctionsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Disallowed Functions",
            code: "twig-disallowed-functions",
            description: indoc! {r"
                Reports calls to the functions listed in `names`.

                Names match exactly, including macros imported under those names. An empty list reports nothing.
            "},
            good_example: indoc! {r"
                {{ title }}
            "},
            bad_example: indoc! {r"
                {{ source('secret.txt') }}
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
        let Node::Call(call) = node else {
            return;
        };

        let Expression::Name(name) = call.callee else {
            return;
        };

        if !std::str::from_utf8(name.name).is_ok_and(|function| self.names.contains(function)) {
            return;
        }

        let function = BytesDisplay(name.name);
        ctx.collector.report(
            Issue::new(self.cfg.level, format!("The `{function}` function is disallowed in this project."))
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(name.span).with_message(format!("Call to `{function}`")))
                .with_note(format!("`{function}` is listed in `[twig.linter.rules.disallowed-functions] names`."))
                .with_help(format!(
                    "Replace `{function}()` with an approach the project allows, or remove `{function}` from the `names` list if it is now permitted.",
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
        rule = DisallowedFunctionsRule,
        settings = |s: &mut Settings| s.rules.disallowed_functions.config.names = vec!["source".into()],
        code = r"{{ custom_function() }}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = DisallowedFunctionsRule,
        settings = |s: &mut Settings| s.rules.disallowed_functions.config.names = vec!["source".into()],
        code = r"{{ object.source() }}",
    }

    test_lint_success! {
        name = valid_template_3,
        rule = DisallowedFunctionsRule,
        settings = |s: &mut Settings| s.rules.disallowed_functions.config.names = vec!["source".into()],
        code = r"{{ source }}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = DisallowedFunctionsRule,
        count = 1,
        settings = |s: &mut Settings| s.rules.disallowed_functions.config.names = vec!["source".into()],
        code = r"{{ source('secret.txt') }}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = DisallowedFunctionsRule,
        count = 1,
        settings = |s: &mut Settings| s.rules.disallowed_functions.config.names = vec!["source".into()],
        code = r"{% from 'macros' import x as source %}{{ source() }}",
    }
}
