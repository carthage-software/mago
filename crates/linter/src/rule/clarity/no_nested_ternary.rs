use indoc::indoc;
use mago_span::Span;
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
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoNestedTernaryRule {
    meta: &'static RuleMeta,
    cfg: NoNestedTernaryConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoNestedTernaryConfig {
    pub level: Level,
}

impl Default for NoNestedTernaryConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoNestedTernaryConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoNestedTernaryRule {
    type Config = NoNestedTernaryConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Nested Ternary",
            code: "no-nested-ternary",
            description: indoc! {"
                Nested ternary expressions are disallowed to improve code clarity and prevent potential bugs arising from confusion over operator associativity.

                In PHP 8.0 and later, the ternary operator (`? :`) is non-associative. Before PHP 8.0, it was left-associative, which is now deprecated. Most other programming languages treat it as right-associative. This inconsistency across versions and languages can make nested ternaries hard to reason about, even when using parentheses.
            "},
            good_example: indoc! {r#"
                <?php

                if ($user->isAdmin()) {
                    $allowed = true;
                } else {
                    $allowed = $user->isEditor();
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                $allowed = $user->isAdmin() ? true : ($user->isEditor() ? true : false);
            "#},
            category: Category::Clarity,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Conditional];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Conditional(expr) = node else {
            return;
        };

        let Some(then) = expr.then else {
            // we don't care about elvis operator
            return;
        };

        self.check_for_nested(ctx, expr.question_mark.span(), expr.condition);
        self.check_for_nested(ctx, expr.question_mark.span(), then);
        self.check_for_nested(ctx, expr.question_mark.span(), expr.r#else);
    }
}

impl NoNestedTernaryRule {
    fn check_for_nested<'arena>(
        &self,
        ctx: &mut LintContext<'_, 'arena>,
        outer_op_span: Span,
        expr: &'arena Expression<'arena>,
    ) {
        match expr {
            Expression::Parenthesized(Parenthesized { expression, .. }) => {
                self.check_for_nested(ctx, outer_op_span, expression);
            }
            Expression::Conditional(nested) => {
                // If the nested is an elvis operator, we ignore it
                if nested.then.is_none() {
                    return;
                }

                self.report_issue(ctx, outer_op_span, expr.span());
            }
            _ => {}
        }
    }

    fn report_issue(&self, ctx: &mut LintContext, outer_op_span: Span, inner_op_span: Span) {
        let issue = Issue::new(
            self.cfg.level,
            "Nested ternary expressions are confusing due to PHP's operator associativity rules.",
        )
        .with_code(self.meta.code)
        .with_annotation(
            Annotation::primary(inner_op_span).with_message("This is the nested ternary operation"),
        )
        .with_annotation(
            Annotation::secondary(outer_op_span)
                .with_message("It is nested inside this outer ternary operator"),
        )
        .with_note(
            "The ternary operator `? :` is non-associative in PHP 8+, was left-associative before, and is right-associative in most other languages.",
        )
        .with_note(
            "This can lead to unexpected behavior, and even with parentheses, it can be hard to read and understand.",
        )
        .with_help(
            "Consider refactoring the logic into a separate `if/else` statement or a `match` expression for clarity.",
        );

        ctx.collector.report(issue);
    }
}
