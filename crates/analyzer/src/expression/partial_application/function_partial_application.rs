use std::borrow::Cow;

use mago_atom::AtomMap;
use mago_atom::ascii_lowercase_atom;
use mago_atom::atom;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::cast::cast_atomic_to_callable;
use mago_codex::ttype::expander::get_signature_of_function_like_identifier;
use mago_codex::ttype::get_mixed_closure;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionPartialApplication;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::partial_application::create_closure_from_partial_application;
use crate::invocation::Invocation;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::InvocationTarget;
use crate::invocation::InvocationTargetParameter;
use crate::invocation::analyzer::analyze_invocation;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for FunctionPartialApplication<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let callables: Vec<TCallable> =
            resolve_function_callable_types(context, block_context, artifacts, self.function)?
                .into_iter()
                .map(std::borrow::Cow::into_owned)
                .collect();

        let resulting_type = if self.argument_list.is_first_class_callable() {
            let callable_types: Vec<TAtomic> = callables
                .into_iter()
                .map(|mut callable| {
                    if let TCallable::Signature(ref mut sig) = callable {
                        sig.is_closure = true;
                    }

                    TAtomic::Callable(callable)
                })
                .collect();

            if callable_types.is_empty() { get_mixed_closure() } else { TUnion::from_vec(callable_types) }
        } else {
            let mut closure_types = Vec::new();
            for callable in callables {
                let (identifier, signature) = match &callable {
                    TCallable::Alias(id) => {
                        let Some(sig) = get_signature_of_function_like_identifier(id, context.codebase) else {
                            continue;
                        };

                        (*id, sig)
                    }
                    TCallable::Signature(sig) => {
                        if let Some(source_id) = sig.get_source() {
                            (source_id, sig.clone())
                        } else {
                            continue;
                        }
                    }
                };

                let Some(metadata) = context.codebase.get_function_like(&identifier) else {
                    continue;
                };

                // Get parameters from metadata before creating InvocationTarget
                let original_parameters: Vec<_> =
                    metadata.parameters.iter().map(InvocationTargetParameter::FunctionLike).collect();

                let invocation_target = InvocationTarget::FunctionLike {
                    identifier,
                    metadata,
                    inferred_return_type: None,
                    method_context: None,
                    span: self.function.span(),
                };

                let invocation = Invocation::new(
                    invocation_target,
                    InvocationArgumentsSource::PartialArgumentList(&self.argument_list),
                    self.span(),
                );

                let mut template_result = TemplateResult::default();
                let mut parameter_types = AtomMap::default();

                analyze_invocation(
                    context,
                    block_context,
                    artifacts,
                    &invocation,
                    None,
                    &mut template_result,
                    &mut parameter_types,
                )?;

                closure_types.push(create_closure_from_partial_application(
                    &signature,
                    &self.argument_list,
                    &original_parameters,
                    &template_result,
                    context.codebase,
                ));
            }

            if closure_types.is_empty() { get_mixed_closure() } else { TUnion::from_vec(closure_types) }
        };

        artifacts.set_expression_type(self, resulting_type);

        Ok(())
    }
}

fn resolve_function_callable_types<'ctx, 'arena, 'artifacts>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &'artifacts mut AnalysisArtifacts,
    expression: &Expression<'arena>,
) -> Result<Vec<Cow<'artifacts, TCallable>>, AnalysisError> {
    if let Expression::Identifier(function_name) = expression {
        let name = atom(context.resolved_names.get(function_name));
        let unqualified_name = atom(function_name.value());

        let identifier = if context.codebase.function_exists(&name) {
            FunctionLikeIdentifier::Function(name)
        } else if !function_name.is_fully_qualified()
            && unqualified_name != name
            && context.codebase.function_exists(&unqualified_name)
        {
            FunctionLikeIdentifier::Function(unqualified_name)
        } else {
            let issue = if unqualified_name == name {
                Issue::error(format!("Function `{name}` could not be found.")).with_annotation(
                    Annotation::primary(expression.span())
                        .with_message(format!("Undefined function `{name}` called here")),
                )
            } else {
                Issue::error(format!(
                    "Could not find definition for function `{name}` (also tried as `{unqualified_name}` in a broader scope)."
                )).with_annotation(
                    Annotation::primary(expression.span()).with_message(format!("Attempted to use function `{name}` which is undefined")),
                ).with_note(
                    format!("Neither `{name}` (e.g., in current namespace) nor `{unqualified_name}` (e.g., global fallback) could be resolved."),
                )
            };

            context.collector.report_with_code(
                IssueCode::NonExistentFunction,
                issue.with_note("This often means the function is misspelled, not imported correctly (e.g., missing `use` statement for namespaced functions), or not defined/autoloaded.")
                    .with_help(format!("Check for typos in `{name}`. Verify namespace imports if applicable, and ensure the function is defined and accessible."))
            );

            return Ok(vec![]);
        };

        if let FunctionLikeIdentifier::Function(function_name) = identifier {
            artifacts.symbol_references.add_reference_to_symbol(
                &block_context.scope,
                ascii_lowercase_atom(function_name.as_ref()),
                false,
            );
        }

        return Ok(vec![Cow::Owned(TCallable::Alias(identifier))]);
    }

    let was_inside_call = block_context.flags.inside_call();
    block_context.flags.set_inside_call(true);
    expression.analyze(context, block_context, artifacts)?;
    block_context.flags.set_inside_call(was_inside_call);

    let Some(expression_type) = artifacts.get_expression_type(expression) else {
        return Ok(vec![]);
    };

    let mut targets = vec![];
    for atomic in expression_type.types.as_ref() {
        let as_callable = cast_atomic_to_callable(atomic, context.codebase, None);

        let Some(callable) = as_callable else {
            let type_name = atomic.get_id();

            context.collector.report_with_code(
                IssueCode::InvalidCallable,
                Issue::error(format!(
                    "Expression of type `{type_name}` cannot be treated as a callable.",
                ))
                .with_annotation(
                    Annotation::primary(expression.span())
                        .with_message(format!("This expression (type `{type_name}` ) is not a valid callable"))
                )
                .with_note("To be callable, an expression must resolve to a function name (string), a Closure, an invocable object (object with `__invoke` method), or an array representing a static/instance method.")
                .with_help("Ensure the expression evaluates to a callable type. If it's a variable, check its assigned type. If it's a string, ensure it's a defined function name or valid callable array syntax.".to_string()),
            );

            continue;
        };

        targets.push(callable);
    }

    Ok(targets)
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = closure_creation_carries_func_metadata,
        code = indoc! {r#"
            <?php

            function strlen(string $_string): int { return 1; }

            (((((((((strlen(...)))(...))(...))(...))(...))(...))(...))(...))(...)(str: "hello");
        "#},
        issues = [
            IssueCode::InvalidNamedArgument,
        ],
    }

    test_analysis! {
        name = closure_creation_carries_happy_path,
        code = indoc! {r#"
            <?php

            function strlen(string $_string): int { return 1; }

            (((((((((strlen(...)))(...))(...))(...))(...))(...))(...))(...))(...)(_string: "hello");
        "#},
    }

    test_analysis! {
        name = closure_creation_carries_templates,
        code = indoc! {r"
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

            /**
             * @template T
             * @param list<T> $list
             * @return list<array{T, T}>
             */
            function duplicates_list(array $list): array
            {
                return map_list($list, to_tuple(...));
            }

            function i_take_int(int $_i): void
            {
            }

            $integers = [1, 2, 3, 4, 5];
            $tuples = duplicates_list($integers);

            foreach ($tuples as $tuple) {
                i_take_int($tuple[0]);
                i_take_int($tuple[1]);
                i_take_int($tuple); // error.
            }
        "},
        issues = [
            IssueCode::InvalidArgument, // `$tuple` is a tuple/list, not an `int`.
        ],
    }
}
