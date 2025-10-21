use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
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
pub struct UseCompoundAssignmentRule {
    meta: &'static RuleMeta,
    cfg: UseCompoundAssignmentConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct UseCompoundAssignmentConfig {
    pub level: Level,
}

impl Default for UseCompoundAssignmentConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for UseCompoundAssignmentConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for UseCompoundAssignmentRule {
    type Config = UseCompoundAssignmentConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Use Compound Assignment",
            code: "use-compound-assignment",
            description: indoc! {"
                Enforces the use of compound assignment operators (e.g., `+=`, `.=`)
                over their more verbose equivalents (`$var = $var + ...`).

                Using compound assignments is more concise and idiomatic. For string
                concatenation (`.=`), it can also be more performant as it avoids
                creating an intermediate copy of the string.
            "},
            good_example: indoc! {r#"
                <?php

                $count += 1;
                $message .= ' Hello';
            "#},
            bad_example: indoc! {r#"
                <?php

                $count = $count + 1;
                $message = $message . ' Hello';
            "#},
            category: Category::BestPractices,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Assignment];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Assignment(assignment) = node else {
            return;
        };

        if !assignment.operator.is_assign() {
            return;
        }

        let Expression::Binary(binary) = assignment.rhs else {
            return;
        };

        let Some(compound_op_str) = get_compound_operator(&binary.operator) else {
            return;
        };

        let assignment_lhs_text = &ctx.source_file.contents[assignment.lhs.span().to_range_usize()];
        let binary_lhs_text = &ctx.source_file.contents[binary.lhs.span().to_range_usize()];

        if assignment_lhs_text != binary_lhs_text {
            return;
        }

        let issue = Issue::new(self.cfg.level(), "Use a compound assignment for clarity and performance.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(assignment.span())
                    .with_message("This can be written more concisely"),
            )
            .with_note("Using operators like `+=` or `.=` is more idiomatic and can be faster, especially for string concatenation.")
            .with_help(format!("Use the `{}` operator instead.", compound_op_str));

        ctx.collector.propose(issue, |plan| {
            let replacement_span = assignment.operator.span().join(binary.operator.span());

            plan.replace(replacement_span.to_range(), compound_op_str, SafetyClassification::Safe);
        });
    }
}

const fn get_compound_operator(op: &BinaryOperator) -> Option<&'static str> {
    Some(match op {
        BinaryOperator::Addition(_) => "+=",
        BinaryOperator::Subtraction(_) => "-=",
        BinaryOperator::Multiplication(_) => "*=",
        BinaryOperator::Division(_) => "/=",
        BinaryOperator::Modulo(_) => "%=",
        BinaryOperator::Exponentiation(_) => "**=",
        BinaryOperator::StringConcat(_) => ".=",
        BinaryOperator::BitwiseAnd(_) => "&=",
        BinaryOperator::BitwiseOr(_) => "|=",
        BinaryOperator::BitwiseXor(_) => "^=",
        BinaryOperator::LeftShift(_) => "<<=",
        BinaryOperator::RightShift(_) => ">>=",
        BinaryOperator::NullCoalesce(_) => "??=",
        _ => return None,
    })
}
