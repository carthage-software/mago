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
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoBooleanLiteralComparisonRule {
    meta: &'static RuleMeta,
    cfg: NoBooleanLiteralComparisonConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoBooleanLiteralComparisonConfig {
    pub level: Level,
}

impl Default for NoBooleanLiteralComparisonConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for NoBooleanLiteralComparisonConfig {
    fn default_enabled() -> bool {
        // This rule is disabled by default due to its deprecation.
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoBooleanLiteralComparisonRule {
    type Config = NoBooleanLiteralComparisonConfig;

    fn deprecated() -> bool {
        // The functionality of this rule has been merged into the `analyzer`,
        // preventing false-positives for things like `false === $foo` where `$foo` might be `false|string`
        // for example, and a direct comparison is necessary.
        true
    }

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Boolean Literal Comparison",
            code: "no-boolean-literal-comparison",
            description: indoc! {r#"
                Disallows comparisons where a boolean literal is used as an operand.

                Comparing with a boolean literal (`true` or `false`) is redundant and can often be simplified.
                For example, `if ($x === true)` is equivalent to the more concise `if ($x)`, and
                `if ($y !== false)` is the same as `if ($y)`.
            "#},
            good_example: indoc! {r#"
                <?php

                if ($x) { /* ... */ }
                if (!$y) { /* ... */ }
            "#},
            bad_example: indoc! {r#"
                <?php

                if ($x === true) { /* ... */ }
                if ($y != false) { /* ... */ }
            "#},
            category: Category::Correctness,

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

        let is_negated_comparison = match binary.operator {
            BinaryOperator::Equal(_) | BinaryOperator::Identical(_) => false,
            BinaryOperator::NotEqual(_) | BinaryOperator::NotIdentical(_) | BinaryOperator::AngledNotEqual(_) => true,
            _ => return,
        };

        let Some(literal_val) = get_boolean_literal(binary.lhs).or_else(|| get_boolean_literal(binary.rhs)) else {
            return;
        };

        let literal_str = if literal_val { "true" } else { "false" };

        let help_message = match (is_negated_comparison, literal_val) {
            // `== true` or `=== true` -> `$var`
            (false, true) => "Simplify this expression by using the variable directly, e.g., `if ($variable)`.",
            // `== false` or `=== false` -> `!$var`
            (false, false) => "Simplify this by using the logical NOT operator, e.g., `if (!$variable)`.",
            // `!= true` or `!== true` -> `!$var`
            (true, true) => "Simplify this by using the logical NOT operator, e.g., `if (!$variable)`.",
            // `!= false` or `!== false` -> `$var`
            (true, false) => "Simplify this expression by using the variable directly, e.g., `if ($variable)`.",
        };

        let issue = Issue::new(self.cfg.level, "Avoid direct comparison with boolean literals.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(binary.span())
                    .with_message(format!("This comparison with `{}` is redundant", literal_str)),
            )
            .with_note("Comparing a value directly to `true` or `false` is verbose and can be simplified.")
            .with_help(help_message);

        ctx.collector.report(issue);
    }
}

/// Attempts to extract a boolean literal from an expression, looking through parentheses.
fn get_boolean_literal<'ast, 'arena>(expr: &'ast Expression<'arena>) -> Option<bool> {
    match expr {
        Expression::Literal(Literal::True(_)) => Some(true),
        Expression::Literal(Literal::False(_)) => Some(false),
        Expression::Parenthesized(Parenthesized { expression, .. }) => get_boolean_literal(expression),
        _ => None,
    }
}
