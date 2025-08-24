use std::rc::Rc;

use mago_codex::ttype::TType;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::get_never;
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

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Throw<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let was_inside_throw = block_context.inside_throw;
        block_context.inside_throw = true;
        self.exception.analyze(context, block_context, artifacts)?;
        block_context.inside_throw = was_inside_throw;
        block_context.has_returned = true;
        if let Some(scope) = block_context.finally_scope.as_ref() {
            let mut finally_scope = scope.borrow_mut();

            for (variable, previous_type) in block_context.locals.iter() {
                match finally_scope.locals.get_mut(variable) {
                    Some(finally_type) => {
                        let resulting_type =
                            combine_union_types(previous_type.as_ref(), finally_type.as_ref(), context.codebase, false);

                        finally_scope.locals.insert(variable.clone(), Rc::new(resulting_type));
                    }
                    None => {
                        let mut resulting_type = (**previous_type).clone();
                        resulting_type.possibly_undefined_from_try = true;

                        finally_scope.locals.insert(variable.clone(), Rc::new(resulting_type));
                    }
                };
            }
        }

        if let Some(exception_type) = artifacts.get_expression_type(self.exception) {
            for exception_atomic in exception_type.types.as_ref() {
                if exception_atomic.extends_or_implements(context.codebase, "Throwable") {
                    for object_name in exception_atomic.get_all_object_names() {
                        block_context.possibly_thrown_exceptions.entry(object_name).or_default().insert(self.span());
                    }
                } else {
                    let exception_atomic_str = exception_atomic.get_id();

                    context.collector.report_with_code(
                        IssueCode::InvalidThrow,
                        Issue::error(format!(
                            "Cannot throw type `{exception_atomic_str}` because it is not an instance of Throwable."
                        ))
                        .with_annotation(
                            Annotation::primary(self.span())
                                .with_message(format!("This has type `{exception_atomic_str}`, not `Throwable`"))
                        )
                        .with_note(
                            "Only objects that implement the `Throwable` interface (like `Exception` or `Error`) can be thrown."
                        )
                        .with_help(
                            "Ensure the value being thrown is an instance of `Exception`, `Error`, or a subclass thereof."
                        ),
                    );
                }
            }
        }

        artifacts.set_expression_type(self, get_never());

        Ok(())
    }
}
