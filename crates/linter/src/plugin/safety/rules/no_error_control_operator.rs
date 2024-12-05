use fennec_ast::*;
use fennec_fixer::SafetyClassification;
use fennec_reporting::*;
use fennec_span::HasSpan;
use fennec_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoErrorControlOperatorRule;

impl Rule for NoErrorControlOperatorRule {
    fn get_name(&self) -> &'static str {
        "no-error-control-operator"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Error)
    }
}

impl<'a> Walker<LintContext<'a>> for NoErrorControlOperatorRule {
    fn walk_in_unary_prefix<'ast>(&self, unary_prefix: &'ast UnaryPrefix, context: &mut LintContext<'a>) {
        if let UnaryPrefixOperator::ErrorControl(_) = unary_prefix.operator {
            let issue = Issue::new(context.level(), "unsafe use of error control operator")
                .with_annotation(Annotation::primary(unary_prefix.operator.span()))
                .with_annotation(Annotation::secondary(unary_prefix.operand.span()))
                .with_note("error control operator hide potential errors and make debugging more difficult.")
                .with_help("remove the `@` and use `set_error_handler` to handle errors instead.");

            context.report_with_fix(issue, |plan| {
                plan.delete(unary_prefix.operator.span().to_range(), SafetyClassification::Safe)
            });
        }
    }
}
