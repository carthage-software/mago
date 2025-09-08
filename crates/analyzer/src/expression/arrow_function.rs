use ahash::HashSet;

use mago_codex::context::ScopeContext;
use mago_codex::get_closure;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::get_signature_of_function_like_metadata;
use mago_codex::ttype::union::TUnion;
use mago_span::HasSpan;
use mago_syntax::ast::ArrowFunction;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::heuristic;
use crate::statement::function_like::FunctionLikeBody;
use crate::statement::function_like::analyze_function_like;
use crate::utils::expression::variable::get_variables_referenced_in_expression;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for ArrowFunction<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let s = self.span();

        let Some(function_metadata) = get_closure(context.codebase, &s.file_id, &s.start) else {
            return Err(AnalysisError::InternalError(
                format!(
                    "Metadata for arrow function defined in `{}` at offset {} not found.",
                    context.source_file.name, s.start.offset
                ),
                s,
            ));
        };

        let mut scope = ScopeContext::new();
        scope.set_function_like(Some(function_metadata));
        scope.set_class_like(block_context.scope.get_class_like());
        scope.set_static(self.r#static.is_some());

        let mut inner_block_context = BlockContext::new(scope);

        let variables = get_variables_referenced_in_expression(self.expression, true);
        let params = self.parameter_list.parameters.iter().map(|param| param.variable.name).collect::<HashSet<_>>();

        for (variable, _) in variables {
            if params.contains(&variable) {
                continue;
            }

            if inner_block_context.variables_possibly_in_scope.contains(variable) {
                continue;
            }

            block_context.add_conditionally_referenced_variable(variable);

            if let Some(existing_type) = block_context.locals.get(variable).cloned() {
                inner_block_context.locals.insert(variable.to_string(), existing_type);
            }

            inner_block_context.variables_possibly_in_scope.insert(variable.to_string());
        }

        let inferred_parameter_types = artifacts.inferred_parameter_types.take();
        let inner_artifacts = analyze_function_like(
            context,
            artifacts,
            &mut inner_block_context,
            function_metadata,
            &self.parameter_list,
            FunctionLikeBody::Expression(self.expression),
            inferred_parameter_types,
        )?;

        let function_identifier = FunctionLikeIdentifier::Closure(s.file_id, s.start);

        let resulting_closure =
            if function_metadata.template_types.is_empty() && !inner_artifacts.inferred_return_types.is_empty() {
                let mut signature = get_signature_of_function_like_metadata(
                    &function_identifier,
                    function_metadata,
                    context.codebase,
                    &TypeExpansionOptions::default(),
                );

                let mut inferred_return_type = None;
                for inferred_return in inner_artifacts.inferred_return_types {
                    inferred_return_type = Some(add_optional_union_type(
                        (*inferred_return).clone(),
                        inferred_return_type.as_ref(),
                        context.codebase,
                    ));
                }

                if let Some(inferred_return_type) = inferred_return_type {
                    signature.return_type = Some(Box::new(inferred_return_type));
                }

                TUnion::from_atomic(TAtomic::Callable(TCallable::Signature(signature)))
            } else {
                TUnion::from_atomic(TAtomic::Callable(TCallable::Alias(function_identifier)))
            };

        artifacts.set_expression_type(self, resulting_closure);

        heuristic::check_function_like(
            function_metadata,
            self.parameter_list.parameters.as_slice(),
            FunctionLikeBody::Expression(self.expression),
            context,
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::test_analysis;

    test_analysis! {
        name = concat_operator_test,
        code = indoc! {r#"
            <?php

            function i_take_float(float $_f): void {}
            function i_take_string(string $_s): void {}

            /**
             * @template T
             * @template U
             *
             * @param list<T> $list
             * @param (Closure(T): U) $callback
             *
             * @return list<U>
             */
            function map_vector(array $list, Closure $callback): array
            {
                $result = [];
                foreach ($list as $item) {
                    $result[] = $callback($item);
                }

                return $result;
            }

            $integers = [1, 2, 3];
            $strings = map_vector($integers, fn(int $i): string => (string) $i);
            $flaots = map_vector($integers, fn(int $i): float => (float) $i);

            foreach ($strings as $s) {
                i_take_string($s);
            }

            foreach ($flaots as $f) {
                i_take_float($f);
            }
        "#}
    }

    test_analysis! {
        name = returns_typed_closure_arrow,
        code = indoc! {r#"
            <?php

            /**
             * @param (Closure(int): int) $f
             * @param (Closure(int): int) $g
             *
             * @return (Closure(int): int)
             */
            function foo(Closure $f, Closure $g): Closure {
                return fn(int $x): int => $f($g($x));
            }
        "#}
    }

    test_analysis! {
        name = inferred_arrow_function_return_type,
        code = indoc! {r#"
            <?php

            /**
             * @param (Closure(): 'Hello, World!') $fn
             */
            function x(Closure $fn)
            {
                echo $fn();
            }

            x(fn(): string => 'Hello, World!');
            x(fn() => 'Hello, World!');
        "#}
    }

    test_analysis! {
        name = arrow_function_returns_never,
        code = indoc! {r#"
            <?php

            function i_never_return(): never {
                while (true) {
                    // Infinite loop
                }
            }

            /**
             * @param (Closure(): never) $task
             * @return never
             */
            function run(Closure $task): never {
                $task();
            }

            run(fn(): never => i_never_return());
        "#}
    }

    test_analysis! {
        name = arrow_function_templates,
        code = indoc! {r#"
            <?php

            function i_take_int(int $_i): void {}
            function i_take_float(float $_f): void {}
            function i_take_string(string $_s): void {}

            /**
             * @template T
             * @template U
             *
             * @param list<T> $list
             * @param (Closure(T): U) $callback
             *
             * @return list<U>
             */
            function map_vector(array $list, Closure $callback): array {
                $result = [];
                foreach ($list as $item) {
                    $result[] = $callback($item);
                }
                return $result;
            }

            /**
             * @template T
             * @template U
             *
             * @param T $item
             * @param (Closure(T): U) $callback
             *
             * @return array{'before': T, 'after': U}
             */
            function cap(mixed $item, Closure $callback): array {
                return ['before' => $item, 'after' => $callback($item)];
            }

            $mapper =
                /**
                 * @template T
                 * @template U
                 *
                 * @param list<T> $list
                 * @param (Closure(T): U) $callback
                 *
                 * @return list<array{'before': T, 'after': U}>
                 */
                fn(array $list, Closure $callback): array => map_vector(
                    $list,
                    /**
                     * @param T $item
                     * @return array{'before': T, 'after': U}
                     */
                    fn($item) => cap($item, $callback),
                );

            $integers = [1, 2, 3];
            foreach ($mapper($integers, fn(int $i): float => (float) $i) as $item) {
                i_take_int($item['before']);
                i_take_float($item['after']);
            }

            foreach ($mapper($integers, fn(int $i): string => (string) $i) as $item) {
                i_take_int($item['before']);
                i_take_string($item['after']);
            }
        "#}
    }
}
