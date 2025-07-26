use mago_codex::ttype::TType;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::get_int;
use mago_codex::ttype::get_iterable_parameters;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::get_null;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::issue::TypingIssueKind;

impl Analyzable for Yield {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        match self {
            Yield::Value(yield_value) => yield_value.analyze(context, block_context, artifacts),
            Yield::Pair(yield_pair) => yield_pair.analyze(context, block_context, artifacts),
            Yield::From(yield_from) => yield_from.analyze(context, block_context, artifacts),
        }
    }
}

impl Analyzable for YieldValue {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let key_type = get_int();
        let value_type = if let Some(value) = self.value.as_ref() {
            let was_inside_call = block_context.inside_call;
            block_context.inside_call = true;
            value.analyze(context, block_context, artifacts)?;
            block_context.inside_call = was_inside_call;

            artifacts.get_expression_type(value).cloned().unwrap_or_else(get_mixed_any)
        } else {
            get_null()
        };

        let Some((k, v, s, _)) = get_current_generator_parameters(context, block_context, self.span()) else {
            return Ok(());
        };

        if !union_comparator::is_contained_by(
            context.codebase,
            context.interner,
            &value_type,
            &v,
            false,
            false,
            false,
            &mut ComparisonResult::new(),
        ) {
            context.buffer.report(
                TypingIssueKind::InvalidYieldValueType,
                Issue::error(format!(
                    "Invalid value type yielded; expected `{}`, but found `{}`.",
                    v.get_id(Some(context.interner)),
                    value_type.get_id(Some(context.interner))
                ))
                .with_annotation(
                    Annotation::primary(self.value.as_ref().map_or_else(|| self.span(), |val| val.span()))
                        .with_message(format!("This expression yields type `{}`", value_type.get_id(Some(context.interner)))),
                )
                .with_note("The type of the value yielded must be assignable to the value type declared in the Generator's return type hint.")
                .with_help("Ensure the yielded value matches the expected type, or adjust the Generator's return type hint."),
            );
        }

        if !union_comparator::is_contained_by(
            context.codebase,
            context.interner,
            &key_type,
            &k,
            false,
            false,
            false,
            &mut ComparisonResult::new(),
        ) {
            context.buffer.report(
                TypingIssueKind::InvalidYieldKeyType,
                Issue::error(format!(
                    "Invalid key type yielded implicitly; expected `{}`, but implicit key is `{}`.",
                    k.get_id(Some(context.interner)),
                    key_type.get_id(Some(context.interner))
                ))
                .with_annotation(
                    Annotation::primary(self.span())
                        .with_message(format!("Implicitly yields key of type `{}`", key_type.get_id(Some(context.interner)))),
                )
                .with_note("When `yield $value` is used, an implicit integer key is generated. This key must be assignable to the key type declared in the Generator's return type hint.")
                .with_help("Use `yield $key => $value;` to specify a key of the correct type, or adjust the Generator's key type hint."),
            );
        }

        artifacts.set_expression_type(self, s);

        Ok(())
    }
}

impl Analyzable for YieldPair {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let key_type = {
            let was_inside_call = block_context.inside_call;
            block_context.inside_call = true;
            self.key.analyze(context, block_context, artifacts)?;
            block_context.inside_call = was_inside_call;

            artifacts.get_expression_type(&self.key).cloned().unwrap_or_else(get_mixed_any)
        };

        let value_type = {
            let was_inside_call = block_context.inside_call;
            block_context.inside_call = true;
            self.value.analyze(context, block_context, artifacts)?;
            block_context.inside_call = was_inside_call;

            artifacts.get_expression_type(&self.value).cloned().unwrap_or_else(get_mixed_any)
        };

        let Some((k, v, s, _)) = get_current_generator_parameters(context, block_context, self.span()) else {
            return Ok(());
        };

        if !union_comparator::is_contained_by(
            context.codebase,
            context.interner,
            &value_type,
            &v,
            false,
            false,
            false,
            &mut ComparisonResult::new(),
        ) {
            context.buffer.report(
               TypingIssueKind::InvalidYieldValueType,
               Issue::error(format!(
                   "Invalid value type yielded; expected `{}`, but found `{}`.",
                   v.get_id(Some(context.interner)),
                   value_type.get_id(Some(context.interner))
               ))
               .with_annotation(
                   Annotation::primary(self.value.span())
                       .with_message(format!("This expression yields type `{}`", value_type.get_id(Some(context.interner)))),
               )
               .with_note("The type of the value yielded must be assignable to the value type declared in the Generator's return type hint.")
               .with_help("Ensure the yielded value matches the expected type, or adjust the Generator's return type hint."),
            );
        }

        if !union_comparator::is_contained_by(
            context.codebase,
            context.interner,
            &key_type,
            &k,
            false,
            false,
            false,
            &mut ComparisonResult::new(),
        ) {
            context.buffer.report(
                TypingIssueKind::InvalidYieldKeyType,
                Issue::error(format!(
                    "Invalid key type yielded; expected `{}`, but found `{}`.",
                    k.get_id(Some(context.interner)),
                    key_type.get_id(Some(context.interner))
                ))
                .with_annotation(
                    Annotation::primary(self.key.span())
                        .with_message(format!("This key has type `{}`", key_type.get_id(Some(context.interner)))),
                )
                .with_note("The type of the key yielded must be assignable to the key type declared in the Generator's return type hint.")
                .with_help("Ensure the yielded key matches the expected type, or adjust the Generator's key type hint."),
            );
        }

        artifacts.set_expression_type(self, s);

        Ok(())
    }
}

impl Analyzable for YieldFrom {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
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
            context.buffer.report(
                TypingIssueKind::UnknownYieldFromIteratorType,
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
            let (key, value) = if let Some(generator) = atomic.get_generator_parameters(context.interner) {
                // the iterator is a generator! not only does it have to match key and value,
                // but also `send` type must be compatible with the current generator's `send` type
                if !union_comparator::is_contained_by(
                    context.codebase,
                    context.interner,
                    &s,
                    &generator.2,
                    false,
                    false,
                    false,
                    &mut ComparisonResult::new(),
                ) {
                    context.buffer.report(
                        TypingIssueKind::YieldFromInvalidSendType,
                        Issue::error(format!(
                            "Incompatible `send` type for `yield from`: current generator expects to be sent `{}`, but yielded generator expects `{}`.",
                            s.get_id(Some(context.interner)),
                            generator.2.get_id(Some(context.interner))
                        ))
                        .with_annotation(
                            Annotation::primary(self.iterator.span())
                                .with_message(format!("This generator expects to be sent `{}`", generator.2.get_id(Some(context.interner)))),
                        )
                        .with_note("When using `yield from` with another Generator, the `send` type of the inner generator (Ts') must be a supertype of (or equal to) the `send` type of the outer generator (Ts). This means `Ts <: Ts'`.")
                        .with_help("Ensure the send types are compatible, or adjust the Generator type hints."),
                    );
                }

                (generator.0, generator.1)
            } else if let Some(parameters) = get_iterable_parameters(atomic, context.codebase, context.interner) {
                parameters
            } else {
                context.buffer.report(
                    TypingIssueKind::YieldFromNonIterable,
                    Issue::error(format!(
                        "Cannot `yield from` non-iterable type `{}`.",
                        atomic.get_id(Some(context.interner))
                    ))
                    .with_annotation(Annotation::primary(self.iterator.span()).with_message(format!(
                        "Expression cannot be yielded from; it is of type `{}`",
                        atomic.get_id(Some(context.interner))
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
                context.interner,
                &value,
                &v,
                false,
                false,
                false,
                &mut ComparisonResult::new(),
            ) {
                context.buffer.report(
                    TypingIssueKind::YieldFromInvalidValueType,
                    Issue::error(format!(
                        "Invalid value type from `yield from`: current generator expects to yield `{}`, but the inner iterable yields `{}`.",
                        v.get_id(Some(context.interner)),
                        value.get_id(Some(context.interner))
                    ))
                    .with_annotation(
                        Annotation::primary(self.iterator.span())
                            .with_message(format!("This iterable yields values of type `{}`", value.get_id(Some(context.interner)))),
                    )
                    .with_note("The value type yielded by the inner iterable (Tv') must be assignable to the value type of the current generator (Tv). This means `Tv' <: Tv`.")
                    .with_help("Ensure the inner iterable yields compatible value types, or adjust the current Generator's type hint."),
                );
            }

            if !union_comparator::is_contained_by(
                context.codebase,
                context.interner,
                &key,
                &k,
                false,
                false,
                false,
                &mut ComparisonResult::new(),
            ) {
                context.buffer.report(
                   TypingIssueKind::YieldFromInvalidKeyType,
                   Issue::error(format!(
                       "Invalid key type from `yield from`: current generator expects to yield keys of type `{}`, but the inner iterable yields keys of type `{}`.",
                       k.get_id(Some(context.interner)),
                       key.get_id(Some(context.interner))
                   ))
                   .with_annotation(
                       Annotation::primary(self.iterator.span())
                           .with_message(format!("This iterable yields keys of type `{}`", key.get_id(Some(context.interner)))),
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

fn get_current_generator_parameters<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    yield_span: Span,
) -> Option<(TUnion, TUnion, TUnion, TUnion)> {
    let Some(function) = block_context.scope.get_function_like() else {
        context.buffer.report(
            TypingIssueKind::YieldOutsideFunction,
            Issue::error("`yield` can only be used inside a function or method.")
                .with_annotation(
                    Annotation::primary(yield_span).with_message("`yield` used in an invalid context"),
                )
                .with_note("The `yield` keyword is used to create Generators and can only appear within the body of a function or method.")
                .with_help("Move the `yield` expression into a function or method body. If you are in the global scope, you cannot use `yield` directly."),
        );

        return None;
    };

    let Some(return_type_metadata) = function.get_return_type_metadata() else {
        return Some((get_mixed(), get_mixed(), get_mixed(), get_mixed()));
    };

    let iterable_type = &return_type_metadata.type_union;
    let mut key = None;
    let mut value = None;
    let mut sent = None;
    let mut r#return = None;
    for atomic_iterable in &iterable_type.types {
        match atomic_iterable.get_generator_parameters(context.interner) {
            Some((k, v, s, r)) => {
                key = Some(add_optional_union_type(k, key.as_ref(), context.codebase, context.interner));
                value = Some(add_optional_union_type(v, value.as_ref(), context.codebase, context.interner));
                sent = Some(add_optional_union_type(s, sent.as_ref(), context.codebase, context.interner));
                r#return = Some(add_optional_union_type(r, r#return.as_ref(), context.codebase, context.interner));
            }
            None => match get_iterable_parameters(atomic_iterable, context.codebase, context.interner) {
                Some((k, v)) => {
                    key = Some(add_optional_union_type(k, key.as_ref(), context.codebase, context.interner));
                    value = Some(add_optional_union_type(v, value.as_ref(), context.codebase, context.interner));
                    sent = Some(get_mixed());
                    r#return = Some(get_mixed());
                }
                None => {
                    context.buffer.report(
                        TypingIssueKind::InvalidGeneratorReturnType,
                        Issue::error(format!(
                            "Declared return type `{}` for generator function `{}` is not a valid Generator or iterable type.",
                            iterable_type.get_id(Some(context.interner)),
                            function.get_name().map_or_else(|| "current".to_string(), |id| context.interner.lookup(&id).to_string())
                        ))
                        .with_annotation(
                            Annotation::primary(return_type_metadata.span)
                                .with_message(format!("Declared return type is `{}`", iterable_type.get_id(Some(context.interner)))),
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

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::issue::TypingIssueKind;
    use crate::test_analysis;

    test_analysis! {
        name = yield_array_value,
        code = indoc! {r#"
            <?php

            /**
             * @param array<string, string> $array
             * @return iterable<string>
             */
            function generator(array $array): iterable
            {
                yield $array['key'] ?? 'default';
            }
        "#}
    }

    test_analysis! {
        name = yield_invalid_key,
        code = indoc! {r#"
            <?php

            /**
             * @return iterable<int, string>
             */
            function generator(): iterable
            {
                yield 'key' => 'value';
            }
        "#},
        issues = [
            TypingIssueKind::InvalidYieldKeyType,
        ]
    }

    test_analysis! {
        name = yield_invalid_value,
        code = indoc! {r#"
            <?php

            /**
             * @return iterable<int, string>
             */
            function generator(): iterable
            {
                yield 1 => 42;
            }
        "#},
        issues = [
            TypingIssueKind::InvalidYieldValueType,
        ]
    }

    test_analysis! {
        name = yield_from_invalid_type,
        code = indoc! {r#"
            <?php

            /**
             * @return iterable<int, string>
             */
            function generator(): iterable
            {
                yield from [1, 2, 3];
            }
        "#},
        issues = [
            TypingIssueKind::YieldFromInvalidValueType,
        ]
    }

    test_analysis! {
        name = yield_from_invalid_key,
        code = indoc! {r#"
            <?php

            /**
             * @return iterable<string, string>
             */
            function generator(): iterable
            {
                yield from [1 => 'value'];
            }
        "#},
        issues = [
            TypingIssueKind::YieldFromInvalidKeyType,
        ]
    }

    test_analysis! {
        name = yield_global_scope,
        code = indoc! {r#"
            <?php

            yield 'value';
        "#},
        issues = [
            TypingIssueKind::YieldOutsideFunction,
        ]
    }

    test_analysis! {
        name = yield_from_non_iterable,
        code = indoc! {r#"
            <?php

            /**
             * @return iterable<int, string>
             */
            function generator(): iterable
            {
                yield from 42;
            }
        "#},
        issues = [
            TypingIssueKind::YieldFromNonIterable,
        ]
    }

    test_analysis! {
        name = yield_merge_iterables,
        code = indoc! {r#"
            <?php

            /**
             * @template KLeft
             * @template KRight
             * @template VLeft
             * @template VRight
             *
             * @param iterable<KLeft, VLeft> $lhs
             * @param iterable<KRight, VRight> $rhs
             *
             * @return iterable<KLeft|KRight, VLeft|VRight>
             */
            function merge_iterable(iterable $lhs, iterable $rhs): iterable
            {
                foreach ($lhs as $key => $value) {
                    yield $key => $value;
                }

                foreach ($rhs as $key => $value) {
                    yield $key => $value;
                }
            }

            /**
             * @return iterable<string, string>
             */
            function get_string_string_iterable(): iterable {
                return [
                    'key1' => 'value1',
                    'key2' => 'value2',
                ];
            }

            /**
             * @return iterable<int, string>
             */
            function get_int_string_iterable(): iterable {
                return [
                    1 => 'value1',
                    2 => 'value2',
                ];
            }

            function i_take_string(string $_string): void {}
            function i_take_int_or_string(string|int $_value): void {}

            $merged = merge_iterable(
                get_string_string_iterable(),
                get_int_string_iterable()
            );

            foreach ($merged as $key => $value) {
                i_take_int_or_string($key);
                i_take_string($value);
            }
        "#}
    }

    test_analysis! {
        name = yield_from_generator,
        code = indoc! {r#"
            <?php

            /**
             * @return iterable<int, string>
             */
            function generator(): iterable
            {
                yield from get_string_string_iterable();
            }

            /**
             * @return iterable<string, string>
             */
            function get_string_string_iterable(): iterable {
                return [
                    'key1' => 'value1',
                    'key2' => 'value2',
                ];
            }

            function i_take_string(string $_string): void {}

            foreach (generator() as $key => $value) {
                i_take_string($key);
                i_take_string($value);
            }
        "#},
        issues = [
            TypingIssueKind::YieldFromInvalidKeyType,
            TypingIssueKind::InvalidArgument,
        ]
    }
}
