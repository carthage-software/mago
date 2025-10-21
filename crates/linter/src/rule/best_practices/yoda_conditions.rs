use indoc::indoc;
use mago_fixer::SafetyClassification;
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
pub struct YodaConditionsRule {
    meta: &'static RuleMeta,
    cfg: YodaConditionsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct YodaConditionsConfig {
    pub level: Level,
}

impl Default for YodaConditionsConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for YodaConditionsConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for YodaConditionsRule {
    type Config = YodaConditionsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Yoda Conditions",
            code: "yoda-conditions",
            description: indoc! {"
                This rule enforces the use of \"Yoda\" conditions for comparisons. The variable should always be
                on the right side of the comparison, while the constant, literal, or function call is on the left.
                This prevents the common bug of accidentally using an assignment (`=`) instead of a comparison (`==`),
                which would cause a fatal error in a Yoda condition instead of a silent logical bug.
            "},
            good_example: indoc! {r#"
                <?php

                if ( true === $is_active ) { /* ... */ }
                if ( 5 === $count ) { /* ... */ }
            "#},
            bad_example: indoc! {r#"
                <?php

                // Vulnerable to the accidental assignment bug, e.g., if ($is_active = true).
                if ( $is_active === true ) { /* ... */ }
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

        // Only check equality comparisons
        let is_equality = matches!(
            binary.operator,
            BinaryOperator::Equal(_)
                | BinaryOperator::NotEqual(_)
                | BinaryOperator::Identical(_)
                | BinaryOperator::NotIdentical(_)
                | BinaryOperator::AngledNotEqual(_)
        );

        if !is_equality {
            return;
        }

        let left_is_variable = is_writable_variable(binary.lhs);
        let right_is_constant = is_constant_like(binary.rhs);

        // If variable is on the left and constant is on the right, suggest Yoda condition
        if left_is_variable && right_is_constant {
            let issue = Issue::new(self.cfg.level(), "Use Yoda condition style for safer comparisons")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(binary.operator.span()).with_message("Variable should be on the right side"),
                )
                .with_note("Yoda conditions help prevent accidental assignment bugs where `=` is used instead of `==`")
                .with_help("Move constant/literal to left: `5 === $count`");

            ctx.collector.propose(issue, |plan| {
                let source_code = ctx.source_file.contents.as_ref();

                let right_side_span = binary.rhs.span();
                let right_side_start = right_side_span.start.offset as usize;
                let right_side_end = right_side_span.end.offset as usize;
                let right_side = &source_code[right_side_start..right_side_end];

                let left_side_span = binary.lhs.span();
                let left_side_start = left_side_span.start.offset as usize;
                let left_side_end = left_side_span.end.offset as usize;
                let left_side = &source_code[left_side_start..left_side_end];

                plan.replace(right_side_span.to_range(), left_side, SafetyClassification::Safe);
                plan.replace(left_side_span.to_range(), right_side, SafetyClassification::Safe);
            });
        }
    }
}

/// Check if an expression is "constant-like" (literal, array, or function call)
const fn is_constant_like(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::Literal(_)
            | Expression::ConstantAccess(_)
            | Expression::MagicConstant(_)
            | Expression::Array(_)
            | Expression::LegacyArray(_)
            | Expression::Call(Call::Function(_))
    )
}

const fn is_writable_variable(expr: &Expression) -> bool {
    matches!(expr, Expression::Variable(_) | Expression::Access(_) | Expression::ArrayAccess(_))
}
