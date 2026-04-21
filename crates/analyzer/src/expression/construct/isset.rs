use mago_codex::ttype::get_bool;
use mago_codex::ttype::get_false;
use mago_codex::ttype::get_true;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::Expression;
use mago_syntax::ast::IssetConstruct;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for IssetConstruct<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let mut all_definitely_set = true;
        let mut any_definitely_unset = false;
        for value in &self.values {
            if !is_valid_isset_expression(value) {
                context.collector.report_with_code(
                    IssueCode::InvalidIssetExpression,
                    Issue::error("Cannot use `isset()` on the result of an expression.")
                        .with_annotation(
                            Annotation::primary(value.span()).with_message("This is not a variable or property"),
                        )
                        .with_note("The `isset()` function is designed to check if a variable, property, or array element is set and not null.")
                        .with_help("Consider using `null !== expression` for this check instead."),
                );
            }

            let was_inside_isset = block_context.flags.inside_isset();
            block_context.flags.set_inside_isset(true);
            value.analyze(context, block_context, artifacts)?;
            block_context.flags.set_inside_isset(was_inside_isset);

            if let Some(value_type) = artifacts.get_expression_type(value) {
                if value_type.is_never() || value_type.is_null() {
                    any_definitely_unset = true;
                    continue;
                }

                if value_type.possibly_undefined() || value_type.has_null() || value_type.has_nullable_mixed() {
                    all_definitely_set = false;
                }
            } else {
                all_definitely_set = false;
            }
        }

        let result_type = if any_definitely_unset {
            get_false()
        } else if all_definitely_set {
            get_true()
        } else {
            get_bool()
        };

        artifacts.set_expression_type(self, result_type);

        Ok(())
    }
}

const fn is_valid_isset_expression(expression: &Expression) -> bool {
    match expression {
        Expression::Variable(_) | Expression::Access(_) | Expression::ArrayAccess(_) => true,
        Expression::Assignment(assignment) => assignment.operator.is_assign() && assignment.rhs.is_reference(),
        _ => false,
    }
}
