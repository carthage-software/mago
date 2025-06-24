use std::rc::Rc;

use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator::is_contained_by;
use mago_codex::ttype::get_named_object;
use mago_codex::ttype::get_never;
use mago_codex::ttype::wrap_atomic;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::issue::TypingIssueKind;

impl Analyzable for Throw {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
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
                        let resulting_type = combine_union_types(
                            previous_type.as_ref(),
                            finally_type.as_ref(),
                            context.codebase,
                            context.interner,
                            false,
                        );

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

        if let Some(exception_type) = artifacts.get_expression_type(self.exception.as_ref()) {
            let throwable = get_named_object(context.interner, context.interner.intern("Throwable"), None);

            for exception_atomic in exception_type.types.iter().cloned() {
                let candidate = wrap_atomic(exception_atomic);

                if !is_contained_by(
                    context.codebase,
                    context.interner,
                    &candidate,
                    &throwable,
                    false,
                    false,
                    false,
                    &mut ComparisonResult::new(),
                ) {
                    let candidate_str = candidate.get_id(Some(context.interner));
                    context.buffer.report(
                        TypingIssueKind::InvalidThrow,
                        Issue::error(format!(
                            "Cannot throw type `{candidate_str}` because it is not an instance of Throwable."
                        ))
                        .with_annotation(
                            Annotation::primary(self.span())
                                .with_message(format!("This has type `{candidate_str}`, not `Throwable`"))
                        )
                        .with_note(
                            "Only objects that implement the `Throwable` interface (like `Exception` or `Error`) can be thrown."
                        )
                        .with_help(
                            "Ensure the value being thrown is an instance of `Exception`, `Error`, or a subclass thereof."
                        ),
                    );
                } else if let TAtomic::Object(TObject::Named(named_object)) = candidate.get_single_owned() {
                    block_context.possibly_thrown_exceptions.entry(named_object.name).or_default().insert(self.span());
                }
            }
        }

        artifacts.set_expression_type(self, get_never());

        Ok(())
    }
}
