use indoc::indoc;
use mago_text_edit::TextEdit;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Access;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::Call;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

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

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum YodaConditionsMode {
    /// Require Yoda style: constant/literal on the left, variable on the right.
    #[default]
    Require,
    /// Deny Yoda style: variable on the left, constant/literal on the right.
    Deny,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct YodaConditionsConfig {
    pub level: Level,
    pub mode: YodaConditionsMode,
}

impl Default for YodaConditionsConfig {
    fn default() -> Self {
        Self { level: Level::Help, mode: YodaConditionsMode::Require }
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
                This rule controls the use of \"Yoda\" conditions for comparisons, where the constant, literal,
                or function call appears on the left side and the variable on the right.

                In `require` mode (default), Yoda style is enforced. Placing the constant on the left prevents
                the accidental-assignment bug (`=` instead of `==`), which causes a fatal error rather than a
                silent logical bug in a Yoda condition.

                In `deny` mode, Yoda style is forbidden. The variable must appear on the left for readability.
                When using `deny` mode, consider enabling the `no-assign-in-condition` rule to guard against
                accidental assignments (`=` instead of `==`) that Yoda conditions would otherwise catch.
            "},
            good_example: indoc! {r#"
                <?php

                // configured mode: "require"
                if ( true === $is_active ) { /* ... */ }
                if ( 5 === $count ) { /* ... */ }
            "#},
            bad_example: indoc! {r#"
                <?php

                // configured mode: "require"
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

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
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

        let (message, annotation, reason, help) = match self.cfg.mode {
            YodaConditionsMode::Require if is_writable_variable(binary.lhs) && is_constant_like(binary.rhs) => (
                "Use Yoda condition style for safer comparisons",
                "Variable should be on the right side",
                "Yoda conditions help prevent accidental assignment bugs where `=` is used instead of `==`",
                "Move constant/literal to left: `5 === $count`",
            ),
            YodaConditionsMode::Deny if is_constant_like(binary.lhs) && is_writable_variable(binary.rhs) => (
                "Avoid Yoda condition style",
                "Constant should be on the right side",
                "Yoda conditions are harder to read; prefer `$count === 5`.",
                "Move variable to left: `$count === 5`",
            ),
            _ => return,
        };

        let issue = Issue::new(self.cfg.level(), message)
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(binary.operator.span()).with_message(annotation))
            .with_note(reason)
            .with_help(help);

        ctx.collector.propose(issue, |edits| {
            let source_code = ctx.source_file.contents.as_ref();

            let right_side_span = binary.rhs.span();
            let right_side_start = right_side_span.start_offset() as usize;
            let right_side_end = right_side_span.end_offset() as usize;
            let right_side = &source_code[right_side_start..right_side_end];

            let left_side_span = binary.lhs.span();
            let left_side_start = left_side_span.start_offset() as usize;
            let left_side_end = left_side_span.end_offset() as usize;
            let left_side = &source_code[left_side_start..left_side_end];

            edits.push(TextEdit::replace(right_side_span, left_side));
            edits.push(TextEdit::replace(left_side_span, right_side));
        });
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
            | Expression::Access(Access::ClassConstant(_))
    )
}

const fn is_writable_variable(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::Variable(_)
            | Expression::Access(Access::Property(_) | Access::NullSafeProperty(_) | Access::StaticProperty(_))
            | Expression::ArrayAccess(_)
    )
}
