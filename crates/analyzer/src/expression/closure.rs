use std::rc::Rc;

use ahash::HashMap;

use mago_codex::context::ScopeContext;
use mago_codex::get_closure;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::add_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::get_signature_of_function_like_metadata;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
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
use crate::heuristic;
use crate::statement::function_like::FunctionLikeBody;
use crate::statement::function_like::analyze_function_like;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Closure<'arena> {
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
                    "Metadata for closure defined in `{}` at offset {} not found.",
                    context.source_file.name, s.start.offset
                ),
                s,
            ));
        };

        let mut scope = ScopeContext::new();
        scope.set_class_like(block_context.scope.get_class_like());
        scope.set_function_like(Some(function_metadata));
        scope.set_static(self.r#static.is_some());

        let mut inner_block_context = BlockContext::new(scope);

        let mut variable_spans = HashMap::default();
        if let Some(use_clause) = self.use_clause.as_ref() {
            for use_variable in use_clause.variables.iter() {
                let was_inside_general_use = block_context.inside_general_use;
                block_context.inside_general_use = true;
                use_variable.variable.analyze(context, block_context, artifacts)?;
                block_context.inside_general_use = was_inside_general_use;

                let variable = use_variable.variable.name;
                let variable_span = use_variable.variable.span;

                let is_by_reference = use_variable.ampersand.is_some();

                if let Some(previous_span) = variable_spans.get(&variable) {
                    context.collector.report_with_code(
                        IssueCode::DuplicateClosureUseVariable,
                        Issue::error(format!("Variable `{variable}` is imported multiple times into the closure.",))
                            .with_annotation(
                                Annotation::primary(variable_span)
                                    .with_message(format!("Duplicate import of `{variable}`")),
                            )
                            .with_annotation(
                                Annotation::secondary(*previous_span)
                                    .with_message(format!("Variable `{variable}` was already imported here")),
                            )
                            .with_note(
                                "A variable can only be imported into a closure's scope once via the `use` clause.",
                            )
                            .with_help(format!("Remove the redundant import of `{variable}`.")),
                    );
                }

                if !block_context.has_variable(variable) {
                    context.collector.report_with_code(
                        IssueCode::UndefinedVariableInClosureUse,
                        Issue::error(format!(
                            "Cannot import undefined variable `{variable}` into closure.",
                        ))
                        .with_annotation(
                            Annotation::primary(use_variable.variable.span)
                                .with_message(format!("Variable `{variable}` is not defined in the parent scope")),
                        )
                        .with_note(
                            "Only variables that exist in the scope where the closure is defined can be captured using the `use` keyword."
                        )
                        .with_help(format!(
                            "Ensure `{variable}` is defined and assigned a value in the parent scope before the closure definition, or remove it from the `use` clause.",
                        )),
                    );
                }

                variable_spans.insert(variable, variable_span);

                let mut variable_type =
                    block_context.locals.get(variable).cloned().unwrap_or_else(|| Rc::new(get_mixed()));

                if is_by_reference {
                    let inner_variable_type = Rc::make_mut(&mut variable_type);
                    inner_variable_type.by_reference = true;

                    inner_block_context.references_to_external_scope.insert(variable.to_string());
                }

                inner_block_context.locals.insert(variable.to_string(), variable_type.clone());
                inner_block_context.variables_possibly_in_scope.insert(variable.to_string());

                for (variable_id, variable_type) in block_context.locals.iter() {
                    let Some(stripped_variable_id) = variable_id.strip_prefix(variable) else {
                        continue;
                    };

                    if stripped_variable_id.starts_with('[') || stripped_variable_id.starts_with('-') {
                        inner_block_context.locals.insert(variable_id.to_string(), variable_type.clone());
                        inner_block_context.variables_possibly_in_scope.insert(variable_id.to_string());
                    }
                }
            }
        }

        let inferred_parameter_types = artifacts.inferred_parameter_types.take();
        let inner_artifacts = analyze_function_like(
            context,
            artifacts,
            &mut inner_block_context,
            function_metadata,
            &self.parameter_list,
            FunctionLikeBody::Statements(self.body.statements.as_slice(), self.body.span()),
            inferred_parameter_types,
        )?;

        for referenced_variable in inner_block_context.references_to_external_scope {
            let Some(inner_type) = inner_block_context.locals.remove(&referenced_variable) else {
                continue;
            };

            let variable_type = match block_context.locals.remove(&referenced_variable) {
                Some(existing_type) => {
                    Rc::new(add_union_type(Rc::unwrap_or_clone(inner_type), &existing_type, context.codebase, false))
                }
                None => inner_type,
            };

            block_context.locals.insert(referenced_variable, variable_type);
        }

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
            FunctionLikeBody::Statements(self.body.statements.as_slice(), self.body.span()),
            context,
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = inferred_closure_return_type,
        code = indoc! {r#"
            <?php

            /**
             * @param (Closure(): 'Hello, World!') $fn
             */
            function x(Closure $fn)
            {
                echo $fn();
            }

            x(function (): string { return 'Hello, World!'; });
            x(function () { return 'Hello, World!'; });
        "#}
    }

    test_analysis! {
        name = closure_use,
        code = indoc! {r#"
            <?php

            /**
             * Converts the given value into a tuple.
             *
             * @template T
             *
             * @param T $value
             *
             * @return array{0: T, 1: T}
             */
            function to_tuple(mixed $value): array
            {
                return [$value, $value];
            }

            /**
             * @template T
             * @template U
             *
             * @param list<T> $list
             * @param (Closure(T): U) $callback
             *
             * @return list<U>
             */
            function map_list(array $list, Closure $callback): array
            {
                $result = [];
                foreach ($list as $item) {
                    $result[] = $callback($item);
                }

                return $result;
            }

            function i_take_int(int $_i): void
            {
            }

            $integers = [1, 2, 3, 4, 5];
            $tuples = map_list(
                $integers,
                /**
                 * @param int $value
                 *
                 * @return array{0: int, 1: int}
                 */
                function (mixed $value, $_f = null) use ($integers): array {
                    return [$value, $value];
                },
            );

            foreach ($tuples as $tuple) {
                i_take_int($tuple[0]);
                i_take_int($tuple[1]);
                i_take_int($tuple); // error.
            }
        "#},
        issues = [
            IssueCode::InvalidArgument,
        ]
    }

    test_analysis! {
        name = get_current_closure,
        code = indoc! {r#"
            <?php

            class Closure {
                public static function getCurrent(): Closure
                {
                    exit(0);
                }
            }

            $fibaonacci =
                function (int $n): int {
                    if ($n <= 1) {
                        return $n;
                    }

                    $fibaonacci = Closure::getCurrent();

                    return $fibaonacci($n - 1) + $fibaonacci($n - 2);
                };

            echo $fibaonacci(10);
        "#}
    }

    test_analysis! {
        name = get_current_closure_inside_function,
        code = indoc! {r#"
            <?php

            class Closure {
                public static function getCurrent(): Closure
                {
                    exit(0);
                }
            }

            function fibaonacci(int $n): int {
                if ($n <= 1) {
                    return $n;
                }

                $fibaonacci = Closure::getCurrent();

                return $fibaonacci($n - 1) + $fibaonacci($n - 2);
            }

            echo fibaonacci(10);
        "#},
        issues = [
            IssueCode::InvalidStaticMethodCall,
            IssueCode::ImpossibleAssignment,
            IssueCode::UnevaluatedCode,
        ]
    }

    test_analysis! {
        name = get_current_closure_inside_method,
        code = indoc! {r#"
            <?php

            class Closure {
                public static function getCurrent(): Closure
                {
                    exit(0);
                }
            }

            class Foo {
                public function fibaonacci(int $n): int {
                    if ($n <= 1) {
                        return $n;
                    }

                    $fibaonacci = Closure::getCurrent();

                    return $fibaonacci($n - 1) + $fibaonacci($n - 2);
                }
            }

            echo (new Foo())->fibaonacci(10);
        "#},
        issues = [
            IssueCode::InvalidStaticMethodCall,
            IssueCode::ImpossibleAssignment,
            IssueCode::UnevaluatedCode,
        ]
    }

    test_analysis! {
    name = get_current_closure_in_global_scope,
    code = indoc! {r#"
            <?php

            class Closure {
                public static function getCurrent(): Closure
                {
                    exit(0);
                }
            }

            $_fn = Closure::getCurrent();
        "#},
        issues = [
            IssueCode::InvalidStaticMethodCall,
            IssueCode::ImpossibleAssignment,
        ]
    }
}
