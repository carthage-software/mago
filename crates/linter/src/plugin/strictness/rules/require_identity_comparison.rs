use fennec_ast::ast::*;
use fennec_fixer::SafetyClassification;
use fennec_reporting::*;
use fennec_span::*;
use fennec_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RequireIdentityComparisonRule;

impl Rule for RequireIdentityComparisonRule {
    #[inline]
    fn get_name(&self) -> &'static str {
        "require-identity-comparison"
    }

    #[inline]
    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Warning)
    }
}

impl<'a> Walker<LintContext<'a>> for RequireIdentityComparisonRule {
    fn walk_in_binary<'ast>(&self, binary: &'ast Binary, context: &mut LintContext<'a>) {
        match &binary.operator {
            BinaryOperator::Equal(span) => {
                let issue =
                    Issue::new(context.level(), "use identity comparison `===` instead of equality comparison `==`")
                        .with_annotations([
                            Annotation::primary(*span),
                            Annotation::secondary(binary.lhs.span()),
                            Annotation::secondary(binary.rhs.span()),
                        ])
                        .with_note(
                            "identity comparison `===` checks for both value and type equality, \
                    while equality comparison `==` performs type coercion, which can lead to unexpected results",
                        )
                        .with_help("use `===` to ensure both value and type are equal");

                context
                    .report_with_fix(issue, |plan| plan.replace(span.to_range(), "===", SafetyClassification::Unsafe));
            }
            BinaryOperator::NotEqual(span) => {
                let issue =
                    Issue::new(context.level(), "use identity inequality `!==` instead of inequality comparison `!=`")
                        .with_annotations([
                            Annotation::primary(*span),
                            Annotation::secondary(binary.lhs.span()),
                            Annotation::secondary(binary.rhs.span()),
                        ])
                        .with_note(
                            "identity inequality `!==` checks for both value and type inequality, \
                        while inequality comparison `!=` performs type coercion, which can lead to unexpected results",
                        )
                        .with_help("use `!==` to ensure both value and type are different");

                context
                    .report_with_fix(issue, |plan| plan.replace(span.to_range(), "!==", SafetyClassification::Unsafe));
            }
            _ => {}
        }
    }
}
