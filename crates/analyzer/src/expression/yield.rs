use mago_codex::ttype::TType;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::get_int;
use mago_codex::ttype::get_iterable_parameters;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_null;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Yield<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        match self {
            Yield::Value(yield_value) => yield_value.analyze(context, block_context, artifacts),
            Yield::Pair(yield_pair) => yield_pair.analyze(context, block_context, artifacts),
            Yield::From(yield_from) => yield_from.analyze(context, block_context, artifacts),
        }
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for YieldValue<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let key_type = get_int();
        let value_type = if let Some(value) = self.value.as_ref() {
            let was_inside_call = block_context.inside_call;
            block_context.inside_call = true;
            value.analyze(context, block_context, artifacts)?;
            block_context.inside_call = was_inside_call;

            artifacts.get_expression_type(value).cloned().unwrap_or_else(get_mixed)
        } else {
            get_null()
        };

        let Some((k, v, s, _)) = get_current_generator_parameters(context, block_context, self.span()) else {
            return Ok(());
        };

        if !union_comparator::is_contained_by(
            context.codebase,
            &value_type,
            &v,
            false,
            false,
            false,
            &mut ComparisonResult::new(),
        ) {
            context.collector.report_with_code(
                IssueCode::InvalidYieldValueType,
                Issue::error(format!(
                    "Invalid value type yielded; expected `{}`, but found `{}`.",
                    v.get_id(),
                    value_type.get_id()
                ))
                .with_annotation(
                    Annotation::primary(self.value.as_ref().map_or_else(|| self.span(), |val| val.span()))
                        .with_message(format!("This expression yields type `{}`", value_type.get_id())),
                )
                .with_note("The type of the value yielded must be assignable to the value type declared in the Generator's return type hint.")
                .with_help("Ensure the yielded value matches the expected type, or adjust the Generator's return type hint."),
            );
        }

        if !union_comparator::is_contained_by(
            context.codebase,
            &key_type,
            &k,
            false,
            false,
            false,
            &mut ComparisonResult::new(),
        ) {
            context.collector.report_with_code(
                IssueCode::InvalidYieldKeyType,
                Issue::error(format!(
                    "Invalid key type yielded implicitly; expected `{}`, but implicit key is `{}`.",
                    k.get_id(),
                    key_type.get_id()
                ))
                .with_annotation(
                    Annotation::primary(self.span())
                        .with_message(format!("Implicitly yields key of type `{}`", key_type.get_id())),
                )
                .with_note("When `yield $value` is used, an implicit integer key is generated. This key must be assignable to the key type declared in the Generator's return type hint.")
                .with_help("Use `yield $key => $value;` to specify a key of the correct type, or adjust the Generator's key type hint."),
            );
        }

        artifacts.set_expression_type(self, s);

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for YieldPair<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let key_type = {
            let was_inside_call = block_context.inside_call;
            block_context.inside_call = true;
            self.key.analyze(context, block_context, artifacts)?;
            block_context.inside_call = was_inside_call;

            artifacts.get_expression_type(&self.key).cloned().unwrap_or_else(get_mixed)
        };

        let value_type = {
            let was_inside_call = block_context.inside_call;
            block_context.inside_call = true;
            self.value.analyze(context, block_context, artifacts)?;
            block_context.inside_call = was_inside_call;

            artifacts.get_expression_type(&self.value).cloned().unwrap_or_else(get_mixed)
        };

        let Some((k, v, s, _)) = get_current_generator_parameters(context, block_context, self.span()) else {
            return Ok(());
        };

        if !union_comparator::is_contained_by(
            context.codebase,
            &value_type,
            &v,
            false,
            false,
            false,
            &mut ComparisonResult::new(),
        ) {
            context.collector.report_with_code(
               IssueCode::InvalidYieldValueType,
               Issue::error(format!(
                   "Invalid value type yielded; expected `{}`, but found `{}`.",
                   v.get_id(),
                   value_type.get_id()
               ))
               .with_annotation(
                   Annotation::primary(self.value.span())
                       .with_message(format!("This expression yields type `{}`", value_type.get_id())),
               )
               .with_note("The type of the value yielded must be assignable to the value type declared in the Generator's return type hint.")
               .with_help("Ensure the yielded value matches the expected type, or adjust the Generator's return type hint."),
            );
        }

        if !union_comparator::is_contained_by(
            context.codebase,
            &key_type,
            &k,
            false,
            false,
            false,
            &mut ComparisonResult::new(),
        ) {
            context.collector.report_with_code(
                IssueCode::InvalidYieldKeyType,
                Issue::error(format!(
                    "Invalid key type yielded; expected `{}`, but found `{}`.",
                    k.get_id(),
                    key_type.get_id()
                ))
                .with_annotation(
                    Annotation::primary(self.key.span())
                        .with_message(format!("This key has type `{}`", key_type.get_id())),
                )
                .with_note("The type of the key yielded must be assignable to the key type declared in the Generator's return type hint.")
                .with_help("Ensure the yielded key matches the expected type, or adjust the Generator's key type hint."),
            );
        }

        artifacts.set_expression_type(self, s);

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for YieldFrom<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let was_inside_call = block_context.inside_call;
        block_context.inside_call = true;
        self.iterator.analyze(context, block_context, artifacts)?;
        block_context.inside_call = was_inside_call;

        let Some((k, v, s, _)) = get_current_generator_parameters(context, block_context, self.span()) else {
            return Ok(());
        };

        let Some(iterator_type) = artifacts.get_rc_expression_type(&self.iterator).cloned() else {
            context.collector.report_with_code(
                IssueCode::UnknownYieldFromIteratorType,
                Issue::error("Cannot determine the type of the expression in `yield from`.")
                    .with_annotation(
                        Annotation::primary(self.iterator.span())
                            .with_message("The type of this iterator is unknown"),
                    )
                    .with_note(
                        "`yield from` requires an iterable (array or `Traversable`). Its key, value, send, and return types must be compatible with the current generator."
                    )
                    .with_help(
                        "Ensure the expression has a known iterable type. Check for undefined variables or unresolvable function calls.",
                    ),
            );

            artifacts.set_expression_type(self, get_null());

            return Ok(());
        };

        for atomic in iterator_type.types.iter() {
            let (key, value) = if let Some(generator) = atomic.get_generator_parameters() {
                // the iterator is a generator! not only does it have to match key and value,
                // but also `send` type must be compatible with the current generator's `send` type
                if !union_comparator::is_contained_by(
                    context.codebase,
                    &s,
                    &generator.2,
                    false,
                    false,
                    false,
                    &mut ComparisonResult::new(),
                ) {
                    context.collector.report_with_code(
                        IssueCode::YieldFromInvalidSendType,
                        Issue::error(format!(
                            "Incompatible `send` type for `yield from`: current generator expects to be sent `{}`, but yielded generator expects `{}`.",
                            s.get_id(),
                            generator.2.get_id()
                        ))
                        .with_annotation(
                            Annotation::primary(self.iterator.span())
                                .with_message(format!("This generator expects to be sent `{}`", generator.2.get_id())),
                        )
                        .with_note("When using `yield from` with another Generator, the `send` type of the inner generator (Ts') must be a supertype of (or equal to) the `send` type of the outer generator (Ts). This means `Ts <: Ts'`.")
                        .with_help("Ensure the send types are compatible, or adjust the Generator type hints."),
                    );
                }

                (generator.0, generator.1)
            } else if let Some(parameters) = get_iterable_parameters(atomic, context.codebase) {
                parameters
            } else {
                context.collector.report_with_code(
                    IssueCode::YieldFromNonIterable,
                    Issue::error(format!(
                        "Cannot `yield from` non-iterable type `{}`.",
                        atomic.get_id()
                    ))
                    .with_annotation(Annotation::primary(self.iterator.span()).with_message(format!(
                        "Expression cannot be yielded from; it is of type `{}`",
                        atomic.get_id()
                    )))
                    .with_note(
                        "`yield from` requires an `iterable` (e.g., `array` or an object implementing `Traversable`).",
                    )
                    .with_help("Ensure the expression used with `yield from` always evaluates to an iterable type."),
                );

                continue;
            };

            if !union_comparator::is_contained_by(
                context.codebase,
                &value,
                &v,
                false,
                false,
                false,
                &mut ComparisonResult::new(),
            ) {
                context.collector.report_with_code(
                    IssueCode::YieldFromInvalidValueType,
                    Issue::error(format!(
                        "Invalid value type from `yield from`: current generator expects to yield `{}`, but the inner iterable yields `{}`.",
                        v.get_id(),
                        value.get_id()
                    ))
                    .with_annotation(
                        Annotation::primary(self.iterator.span())
                            .with_message(format!("This iterable yields values of type `{}`", value.get_id())),
                    )
                    .with_note("The value type yielded by the inner iterable (Tv') must be assignable to the value type of the current generator (Tv). This means `Tv' <: Tv`.")
                    .with_help("Ensure the inner iterable yields compatible value types, or adjust the current Generator's type hint."),
                );
            }

            if !union_comparator::is_contained_by(
                context.codebase,
                &key,
                &k,
                false,
                false,
                false,
                &mut ComparisonResult::new(),
            ) {
                context.collector.report_with_code(
                   IssueCode::YieldFromInvalidKeyType,
                   Issue::error(format!(
                       "Invalid key type from `yield from`: current generator expects to yield keys of type `{}`, but the inner iterable yields keys of type `{}`.",
                       k.get_id(),
                       key.get_id()
                   ))
                   .with_annotation(
                       Annotation::primary(self.iterator.span())
                           .with_message(format!("This iterable yields keys of type `{}`", key.get_id())),
                   )
                   .with_note("The key type yielded by the inner iterable (Tk') must be assignable to the key type of the current generator (Tk). This means `Tk' <: Tk`.")
                   .with_help("Ensure the inner iterable yields compatible key types, or adjust the current Generator's type hint."),
                );
            }
        }

        artifacts.set_expression_type(self, get_null());

        Ok(())
    }
}

fn get_current_generator_parameters<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    yield_span: Span,
) -> Option<(TUnion, TUnion, TUnion, TUnion)> {
    let Some(function) = block_context.scope.get_function_like() else {
        context.collector.report_with_code(
            IssueCode::YieldOutsideFunction,
            Issue::error("`yield` can only be used inside a function or method.")
                .with_annotation(
                    Annotation::primary(yield_span).with_message("`yield` used in an invalid context"),
                )
                .with_note("The `yield` keyword is used to create Generators and can only appear within the body of a function or method.")
                .with_help("Move the `yield` expression into a function or method body. If you are in the global scope, you cannot use `yield` directly."),
        );

        return None;
    };

    let Some(return_type_metadata) = &function.return_type_metadata else {
        return Some((get_mixed(), get_mixed(), get_mixed(), get_mixed()));
    };

    let iterable_type = &return_type_metadata.type_union;
    let mut key = None;
    let mut value = None;
    let mut sent = None;
    let mut r#return = None;
    for atomic_iterable in iterable_type.types.as_ref() {
        match atomic_iterable.get_generator_parameters() {
            Some((k, v, s, r)) => {
                key = Some(add_optional_union_type(k, key.as_ref(), context.codebase));
                value = Some(add_optional_union_type(v, value.as_ref(), context.codebase));
                sent = Some(add_optional_union_type(s, sent.as_ref(), context.codebase));
                r#return = Some(add_optional_union_type(r, r#return.as_ref(), context.codebase));
            }
            None => match get_iterable_parameters(atomic_iterable, context.codebase) {
                Some((k, v)) => {
                    key = Some(add_optional_union_type(k, key.as_ref(), context.codebase));
                    value = Some(add_optional_union_type(v, value.as_ref(), context.codebase));
                    sent = Some(get_mixed());
                    r#return = Some(get_mixed());
                }
                None => {
                    context.collector.report_with_code(
                        IssueCode::InvalidGeneratorReturnType,
                        Issue::error(format!(
                            "Declared return type `{}` for generator function `{}` is not a valid Generator or iterable type.",
                            iterable_type.get_id(),
                            function.name.map_or_else(|| "current", |id| id.as_str())
                        ))
                        .with_annotation(
                            Annotation::primary(return_type_metadata.span)
                                .with_message(format!("Declared return type is `{}`", iterable_type.get_id())),
                        )
                        .with_annotation(
                            Annotation::secondary(yield_span)
                                .with_message("`yield` used in a generator function with an invalid return type")
                        )
                        .with_note(
                            "Functions containing `yield` are generators. Their return type hint must be `Generator`, `Iterator`, `Traversable`, or `iterable`."
                        )
                        .with_help(
                            "Adjust the return type hint to a valid Generator signature (e.g., `Generator<K, V, S, R>`) or a compatible iterable type.",
                        ),
                    );

                    return None;
                }
            },
        }
    }

    Some((
        key.unwrap_or_else(get_mixed),
        value.unwrap_or_else(get_mixed),
        sent.unwrap_or_else(get_mixed),
        r#return.unwrap_or_else(get_mixed),
    ))
}
