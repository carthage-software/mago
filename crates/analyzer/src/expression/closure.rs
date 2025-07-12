use std::rc::Rc;

use ahash::HashMap;
use ahash::HashSet;

use mago_codex::context::ScopeContext;
use mago_codex::data_flow::node::DataFlowNode;
use mago_codex::data_flow::node::DataFlowNodeId;
use mago_codex::data_flow::node::DataFlowNodeKind;
use mago_codex::data_flow::node::VariableSourceKind;
use mago_codex::data_flow::path::PathKind;
use mago_codex::get_closure;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::misc::VariableIdentifier;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::get_signature_of_function_like_metadata;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::union::TUnion;
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
use crate::statement::function_like::FunctionLikeBody;
use crate::statement::function_like::analyze_function_like;

impl Analyzable for Closure {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let span = self.span();

        let Some(function_metadata) = get_closure(context.codebase, context.interner, &span.start) else {
            return Err(AnalysisError::InternalError(
                format!(
                    "Metadata for closure defined in `{}` at offset {} not found.",
                    context.interner.lookup(&span.start.source.0),
                    span.start.offset
                ),
                span,
            ));
        };

        let mut scope = ScopeContext::new();
        scope.set_class_like(block_context.scope.get_class_like());
        scope.set_function_like(Some(function_metadata));
        scope.set_static(self.r#static.is_some());

        let mut referenced_variables = HashSet::default();
        let mut imported_variables = HashMap::default();
        let mut variable_spans = HashMap::default();
        if let Some(use_clause) = self.use_clause.as_ref() {
            for use_variable in use_clause.variables.iter() {
                let was_inside_general_use = block_context.inside_general_use;
                block_context.inside_general_use = true;
                use_variable.variable.analyze(context, block_context, artifacts)?;
                block_context.inside_general_use = was_inside_general_use;

                let variable = use_variable.variable.name;
                let variable_span = use_variable.variable.span;
                let variable_str = context.interner.lookup(&variable);

                if let Some(ampersand_span) = use_variable.ampersand.as_ref() {
                    context.buffer.report(
                        TypingIssueKind::UnsupportedReferenceInClosureUse,
                        Issue::warning(format!(
                            "Unsupported by-reference import: Mago does not analyze by-reference captures (`use (&$var)`) for closures like `{variable_str}`.",
                        ))
                        .with_annotation(
                            Annotation::primary(*ampersand_span)
                                .with_message("This by-reference import (`&`) is not supported by Mago's analysis."),
                        )
                        .with_annotation(
                            Annotation::secondary(variable_span)
                                .with_message(format!("Variable `{variable_str}` impacted")),
                        )
                        .with_note(
                            format!("Mago will treat `{variable_str}` as if it were imported by value. This WILL lead to incorrect type tracking and potentially missed errors or false positives if the reference is modified and its state is shared.")
                        )
                        .with_note(
                            "Due to the complexities of tracking by-reference semantics accurately in all cases for closures, Mago is unlikely to support this feature."
                        )
                        .with_help(
                            "Avoid by-reference `use` variables with Mago. For shared mutable state, use an object instead, as objects are always passed by reference in PHP and their state changes can be tracked more reliably."
                        ),
                    );

                    referenced_variables.insert(variable);
                }

                if let Some(previous_span) = variable_spans.get(&variable) {
                    context.buffer.report(
                        TypingIssueKind::DuplicateClosureUseVariable,
                        Issue::error(
                            format!("Variable `{variable_str}` is imported multiple times into the closure.",),
                        )
                        .with_annotation(
                            Annotation::primary(variable_span)
                                .with_message(format!("Duplicate import of `{variable_str}`")),
                        )
                        .with_annotation(
                            Annotation::secondary(*previous_span)
                                .with_message(format!("Variable `{variable_str}` was already imported here")),
                        )
                        .with_note("A variable can only be imported into a closure's scope once via the `use` clause.")
                        .with_help(format!("Remove the redundant import of `{variable_str}`.")),
                    );
                }

                if !block_context.has_variable(variable_str) {
                    context.buffer.report(
                        TypingIssueKind::UndefinedVariableInClosureUse,
                        Issue::error(format!(
                            "Cannot import undefined variable `{variable_str}` into closure.",
                        ))
                        .with_annotation(
                            Annotation::primary(use_variable.variable.span)
                                .with_message(format!("Variable `{variable_str}` is not defined in the parent scope")),
                        )
                        .with_note(
                            "Only variables that exist in the scope where the closure is defined can be captured using the `use` keyword."
                        )
                        .with_help(format!(
                            "Ensure `{variable_str}` is defined and assigned a value in the parent scope before the closure definition, or remove it from the `use` clause.",
                        )),
                    );
                }

                variable_spans.insert(variable, variable_span);

                let mut variable_type = match block_context.locals.remove(variable_str) {
                    Some(existing_type) => existing_type.as_ref().to_owned(),
                    None => get_mixed_any(),
                };

                for parent_node in variable_type.parent_nodes.iter_mut() {
                    artifacts.data_flow_graph.add_path(
                        parent_node,
                        &DataFlowNode {
                            id: DataFlowNodeId::Var(VariableIdentifier(variable), variable_span),
                            kind: DataFlowNodeKind::VariableUseSource {
                                span: variable_span,
                                kind: VariableSourceKind::ClosureUse,
                                pure: false,
                                has_parent_nodes: true,
                                from_loop_init: false,
                            },
                        },
                        PathKind::Default,
                    );
                }

                let rc_type = Rc::new(variable_type);

                block_context.locals.insert(variable_str.to_string(), rc_type.clone());
                imported_variables.insert(variable_str.to_string(), rc_type);
            }
        }

        let (inner_block_context, inner_artifacts) = analyze_function_like(
            context,
            artifacts,
            scope,
            function_metadata,
            &self.parameter_list,
            FunctionLikeBody::Statements(self.body.statements.as_slice()),
            imported_variables,
        )?;

        for referenced_variable in referenced_variables {
            let variable_str = context.interner.lookup(&referenced_variable);
            let Some(inner_variable_type) = inner_block_context.locals.get(variable_str) else {
                block_context.locals.insert(variable_str.to_string(), Rc::new(get_mixed()));

                continue;
            };

            let Some(outer_variable_type) = block_context.locals.get(variable_str) else {
                continue;
            };

            let combined_type = combine_union_types(
                outer_variable_type.as_ref(),
                inner_variable_type.as_ref(),
                context.codebase,
                context.interner,
                false,
            );

            block_context.locals.insert(variable_str.to_string(), Rc::new(combined_type));
        }

        let function_identifier = FunctionLikeIdentifier::Closure(span.start);

        let resulting_closure = if function_metadata.template_types.is_empty() {
            let mut signature = get_signature_of_function_like_metadata(
                &function_identifier,
                function_metadata,
                context.codebase,
                context.interner,
                &TypeExpansionOptions::default(),
            );

            let mut inferred_return_type = None;
            for inferred_return in inner_artifacts.inferred_return_types {
                inferred_return_type = Some(add_optional_union_type(
                    inferred_return,
                    inferred_return_type.as_ref(),
                    context.codebase,
                    context.interner,
                ));
            }

            if let Some(inferred_return_type) = inferred_return_type {
                signature.return_type = Some(Box::new(inferred_return_type));
            }

            TUnion::new(vec![TAtomic::Callable(TCallable::Signature(signature))])
        } else {
            TUnion::new(vec![TAtomic::Callable(TCallable::Alias(function_identifier))])
        };

        artifacts.set_expression_type(self, resulting_closure);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::issue::TypingIssueKind;
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
            TypingIssueKind::InvalidArgument,
        ]
    }
}
