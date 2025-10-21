use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoSprintfConcatRule {
    meta: &'static RuleMeta,
    cfg: NoSprintfConcatConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoSprintfConcatConfig {
    pub level: Level,
}

impl Default for NoSprintfConcatConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoSprintfConcatConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoSprintfConcatRule {
    type Config = NoSprintfConcatConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Sprintf Concat",
            code: "no-sprintf-concat",
            description: indoc! {r#"
                Disallows string concatenation with the result of an `sprintf` call.

                Concatenating with `sprintf` is less efficient and can be less readable than
                incorporating the string directly into the format template. This pattern
                creates an unnecessary intermediate string and can make the final output
                harder to see at a glance.
            "#},
            good_example: indoc! {r#"
                <?php

                $name = 'World';
                $greeting = sprintf('Hello, %s!', $name);
            "#},
            bad_example: indoc! {r#"
                <?php

                $name = 'World';
                $greeting = 'Hello, ' . sprintf('%s!', $name);
            "#},
            category: Category::BestPractices,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Binary];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Binary(binary) = node else {
            return;
        };

        if !matches!(binary.operator, BinaryOperator::StringConcat(_)) {
            return;
        }

        let (sprintf_call_expr, other_expr) = if is_sprintf_call(binary.lhs, ctx) {
            (&binary.lhs, &binary.rhs)
        } else if is_sprintf_call(binary.rhs, ctx) {
            (&binary.rhs, &binary.lhs)
        } else {
            return;
        };

        let issue = Issue::new(self.cfg.level, "String concatenation with `sprintf` can be simplified.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(binary.operator.span()).with_message("This concatenation can be avoided"),
            )
            .with_annotation(
                Annotation::secondary(sprintf_call_expr.span()).with_message("The result of this `sprintf` call..."),
            )
            .with_annotation(
                Annotation::secondary(other_expr.span()).with_message("...is being joined with this expression"),
            )
            .with_note(
                "Combining all parts into a single `sprintf` call is more efficient and makes the code more readable.",
            )
            .with_help("Incorporate the concatenated content into the `sprintf` format argument.");

        ctx.collector.report(issue);
    }
}

fn is_sprintf_call<'arena>(expression: &Expression<'arena>, context: &LintContext<'_, 'arena>) -> bool {
    let Expression::Call(Call::Function(call)) = expression else {
        return false;
    };

    function_call_matches(context, call, "sprintf")
}
