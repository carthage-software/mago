use ahash::HashSet;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::block::BreakContext;
use crate::context::scope::loop_scope::LoopScope;
use crate::error::AnalysisError;
use crate::expression::assignment::assign_to_expression;
use crate::issue::TypingIssueKind;
use crate::statement::r#loop;
use crate::utils::expression::get_expression_id;

impl Analyzable for Foreach {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let mut has_reference = false;
        let mut safe_variable_ids = HashSet::default();
        if let Some(key_expression) = self.target.key() {
            let (key_has_references, key_safe_variables) =
                r#loop::scrape_variables_from_expression(context, key_expression);

            has_reference = key_has_references;
            safe_variable_ids.extend(key_safe_variables);
        }

        let (value_has_references, value_safe_variables) =
            r#loop::scrape_variables_from_expression(context, self.target.value());

        has_reference |= value_has_references;
        safe_variable_ids.extend(value_safe_variables);

        let iterator = self.expression.as_ref();
        has_reference = has_reference
            || if let Expression::UnaryPrefix(UnaryPrefix { operator, .. }) = iterator {
                matches!(operator, UnaryPrefixOperator::Reference(_))
            } else {
                false
            };

        let iterator_variable_id = get_expression_id(
            iterator,
            block_context.scope.get_class_like_name(),
            context.resolved_names,
            context.interner,
            Some(context.codebase),
        );

        let (always_enters_loop, key_type, value_type) =
            r#loop::analyze_iterator(context, block_context, artifacts, iterator, iterator_variable_id.as_ref(), self)?;

        if key_type.is_never() || value_type.is_never() {
            return Ok(());
        }

        let mut loop_block_context = block_context.clone();
        loop_block_context.inside_loop = true;
        loop_block_context.break_types.push(BreakContext::Loop);

        if let Some(key_expression) = self.target.key() {
            let key_expression_id = get_expression_id(
                key_expression,
                block_context.scope.get_class_like_name(),
                context.resolved_names,
                context.interner,
                Some(context.codebase),
            );

            let assigned = assign_to_expression(
                context,
                &mut loop_block_context,
                artifacts,
                key_expression,
                key_expression_id,
                None,
                key_type,
                false,
            )?;

            if !assigned {
                context.buffer.report(
                    TypingIssueKind::InvalidForeachKey,
                    Issue::error("The key expression in `foreach` is not assignable.")
                        .with_annotation(
                            Annotation::primary(key_expression.span())
                                .with_message("This expression cannot be assigned to"),
                        )
                        .with_note("The key expression must be writable.")
                        .with_help(
                            "Ensure the key expression is a valid variable, array, or an object property. If using a complex expression, consider assigning it to a variable first.",
                        ),
                );
            }
        }

        let value_expression = self.target.value();
        let value_expression_id = get_expression_id(
            value_expression,
            block_context.scope.get_class_like_name(),
            context.resolved_names,
            context.interner,
            Some(context.codebase),
        );

        let assigned = assign_to_expression(
            context,
            &mut loop_block_context,
            artifacts,
            value_expression,
            value_expression_id,
            None,
            value_type,
            false,
        )?;

        if !assigned {
            context.buffer.report(
                TypingIssueKind::InvalidForeachValue,
                Issue::error("The value expression in `foreach` is not assignable.")
                    .with_annotation(
                        Annotation::primary(value_expression.span())
                            .with_message("This expression cannot be assigned to"),
                    )
                    .with_note("The value expression must be writable.")
                    .with_help(
                        "Ensure the value expression is a valid variable, array, or an object property. If using a complex expression, consider assigning it to a variable first.",
                    ),
            );
        }

        if has_reference {
            context.buffer.report(
                TypingIssueKind::UnsupportedReferenceOperation,
                Issue::error("Using references in `foreach` is not supported.")
                    .with_annotation(
                        Annotation::primary(self.span()).with_message("References in `foreach` are not supported"),
                    )
                    .with_note("Using references in `foreach` is not yet supported by the analyzer.")
                    .with_help("Consider using a regular variable assignment instead of a reference."),
            );
        }

        let loop_scope = LoopScope::new(self.span(), block_context.locals.clone(), None);

        let (inner_loop_block_context, loop_scope) = r#loop::analyze(
            context,
            self.body.statements(),
            vec![],
            vec![],
            loop_scope,
            &mut loop_block_context,
            block_context,
            artifacts,
            false,
            always_enters_loop,
        )?;

        r#loop::inherit_loop_block_context(
            context,
            block_context,
            loop_block_context,
            inner_loop_block_context,
            loop_scope,
            always_enters_loop,
            /* infinite_loop = */ false,
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::test_analysis;

    test_analysis! {
        name = foreach_basic,
        code = indoc! {r#"
            <?php

            namespace X;

            /**
             * @template T of int|float
             *
             * @param list<T> $numbers
             *
             * @return ($numbers is non-empty-list<T> ? T : null)
             *
             * @pure
             */
            function min(array $numbers): null|float|int
            {
                $min = null;
                foreach ($numbers as $number) {
                    if (null === $min || $number < $min) {
                        $min = $number;
                    }
                }

                return $min;
            }

            /**
             * @template T
             *
             * @param iterable<T> $numbers
             * @param (callable(T): numeric) $numeric_function
             *
             * @return T|null
             */
            function min_by(iterable $numbers, callable $numeric_function): mixed
            {
                $min = null;
                $min_num = null;
                foreach ($numbers as $value) {
                    $value_num = $numeric_function($value);
                    if (null === $min_num || $value_num <= $min_num) {
                        $min = $value;
                        $min_num = $value_num;
                    }
                }

                return $min;
            }

            /**
             * @template T of int|float
             *
             * @param T $first
             * @param T $second
             * @param T ...$rest
             *
             * @return T
             *
             * @pure
             */
            function minva(int|float $first, int|float $second, int|float ...$rest): int|float
            {
                $min = $first < $second ? $first : $second;
                foreach ($rest as $number) {
                    if ($number < $min) {
                        $min = $number;
                    }
                }

                return $min;
            }

            /**
             * @template T of int|float
             *
             * @param list<T> $numbers
             *
             * @return ($numbers is non-empty-list<T> ? T : null)
             *
             * @pure
             */
            function max(array $numbers): null|int|float
            {
                $max = null;
                foreach ($numbers as $number) {
                    if (null === $max || $number > $max) {
                        $max = $number;
                    }
                }

                return $max;
            }

            /**
             * @template T
             *
             * @param iterable<T> $numbers
             * @param (callable(T): numeric) $numeric_function
             *
             * @return T|null
             */
            function max_by(iterable $numbers, callable $numeric_function): mixed
            {
                $max = null;
                $max_num = null;
                foreach ($numbers as $value) {
                    $value_num = $numeric_function($value);
                    if (null === $max_num || $value_num >= $max_num) {
                        $max = $value;
                        $max_num = $value_num;
                    }
                }

                return $max;
            }

            /**
             * @template T of int|float
             *
             * @param T $first
             * @param T $second
             * @param T ...$rest
             *
             * @return T
             *
             * @pure
             */
            function maxva(int|float $first, int|float $second, int|float ...$rest): int|float
            {
                $max = $first > $second ? $first : $second;
                foreach ($rest as $number) {
                    if ($number > $max) {
                        $max = $number;
                    }
                }

                return $max;
            }
        "#}
    }
}
