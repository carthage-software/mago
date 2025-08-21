use mago_codex::ttype::get_bool;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

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
        for value in self.values.iter() {
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

            let was_inside_isset = block_context.inside_isset;
            block_context.inside_isset = true;
            value.analyze(context, block_context, artifacts)?;
            block_context.inside_isset = was_inside_isset;
        }

        artifacts.set_expression_type(self, get_bool());

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
