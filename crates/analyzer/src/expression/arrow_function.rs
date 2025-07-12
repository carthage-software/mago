use std::rc::Rc;

use ahash::HashMap;
use ahash::HashSet;
use mago_codex::context::ScopeContext;
use mago_codex::data_flow::node::DataFlowNode;
use mago_codex::data_flow::node::DataFlowNodeId;
use mago_codex::data_flow::node::DataFlowNodeKind;
use mago_codex::data_flow::path::PathKind;
use mago_codex::get_closure;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::misc::VariableIdentifier;
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
use crate::statement::function_like::FunctionLikeBody;
use crate::statement::function_like::analyze_function_like;
use crate::utils::expression::variable::get_variables_referenced_in_expression;

impl Analyzable for ArrowFunction {
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
                    "Metadata for arrow function defined in `{}` at offset {} not found.",
                    context.interner.lookup(&span.start.source.0),
                    span.start.offset
                ),
                span,
            ));
        };

        let mut scope = ScopeContext::new();
        scope.set_function_like(Some(function_metadata));
        scope.set_class_like(block_context.scope.get_class_like());
        scope.set_static(self.r#static.is_some());

        let mut imported_variables = HashMap::default();
        let variables = get_variables_referenced_in_expression(self.expression.as_ref(), true);
        let parameter_names =
            self.parameter_list.parameters.iter().map(|param| param.variable.name).collect::<HashSet<_>>();

        for (variable, variable_span) in variables {
            if parameter_names.contains(&variable) {
                continue;
            }

            let variable_str = context.interner.lookup(&variable);
            if imported_variables.contains_key(variable_str) {
                continue;
            }

            if !block_context.has_variable(variable_str) {
                continue;
            }

            let Some(existing_type) = block_context.locals.remove(variable_str) else {
                continue;
            };

            let mut variable_type = existing_type.as_ref().to_owned();

            let assignment_node = DataFlowNode {
                id: DataFlowNodeId::Var(VariableIdentifier(variable), variable_span),
                kind: DataFlowNodeKind::VariableUseSink { span: variable_span },
            };

            artifacts.data_flow_graph.add_node(assignment_node.clone());
            let mut parent_nodes = variable_type.parent_nodes.clone();

            if parent_nodes.is_empty() {
                parent_nodes.push(assignment_node);
            } else {
                for parent_node in &parent_nodes {
                    artifacts.data_flow_graph.add_path(parent_node, &assignment_node, PathKind::Default);
                }
            }

            variable_type.parent_nodes = parent_nodes;

            let local_type = Rc::new(variable_type);

            block_context.locals.insert(variable_str.to_string(), local_type.clone());
            imported_variables.insert(variable_str.to_string(), local_type);
        }

        let (_, inner_artifacts) = analyze_function_like(
            context,
            artifacts,
            scope,
            function_metadata,
            &self.parameter_list,
            FunctionLikeBody::Expression(&self.expression),
            imported_variables,
        )?;

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
