use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Hint;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::scope::FunctionLikeScope;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoBooleanFlagParameterRule {
    meta: &'static RuleMeta,
    cfg: NoBooleanFlagParameterConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoBooleanFlagParameterConfig {
    pub level: Level,
    /// When enabled, methods whose name starts with `set` are exempt from this rule.
    pub exclude_setters: bool,
    /// When enabled, constructors are exempt from this rule.
    pub exclude_constructors: bool,
}

impl Default for NoBooleanFlagParameterConfig {
    fn default() -> Self {
        Self { level: Level::Help, exclude_setters: false, exclude_constructors: true }
    }
}

impl Config for NoBooleanFlagParameterConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoBooleanFlagParameterRule {
    type Config = NoBooleanFlagParameterConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Boolean Flag Parameter",
            code: "no-boolean-flag-parameter",
            description: indoc! {r"
                Flags function-like parameters that use a boolean type.

                Boolean flag parameters can indicate a violation of the Single Responsibility Principle (SRP).
                Refactor by extracting the flag logic into its own class or method.
            "},
            good_example: indoc! {r"
                <?php

                function get_difference(string $a, string $b): string {
                    // ...
                }

                function get_difference_case_insensitive(string $a, string $b): string {
                    // ...
                }
            "},
            bad_example: indoc! {r"
                <?php

                function get_difference(string $a, string $b, bool $ignore_case): string {
                    // ...
                }
            "},
            category: Category::Maintainability,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionLikeParameter];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::FunctionLikeParameter(parameter) = node else {
            return;
        };

        if parameter.is_promoted_property() {
            return;
        }

        let Some(Hint::Bool(bool_hint)) = &parameter.hint else {
            return;
        };

        if let Some(FunctionLikeScope::Method(name)) = ctx.scope.get_function_like_scope() {
            if self.cfg.exclude_constructors && name.eq_ignore_ascii_case("__construct") {
                return;
            }

            if self.cfg.exclude_setters && name.len() > 3 && name[..3].eq_ignore_ascii_case("set") {
                return;
            }
        }

        let issue = Issue::new(self.cfg.level, "Avoid boolean flag parameters.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(parameter.variable.span())
                    .with_message("This parameter acts as a boolean flag"),
            )
            .with_annotation(Annotation::secondary(bool_hint.span).with_message("Boolean type declared here"))
            .with_note(
                "Boolean flags often indicate a function has more than one responsibility, making it harder to understand and test.",
            )
            .with_help(
                "Refactor by splitting the function into two separate methods, each with a clear, descriptive name.",
            );

        ctx.collector.report(issue);
    }
}
