use std::rc::Rc;
use std::sync::Arc;

use mago_allocator::Arena;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Foreach;
use mago_syntax::cst::ForeachTarget;
use mago_syntax::cst::UnaryPrefix;
use mago_syntax::cst::UnaryPrefixOperator;
use mago_syntax::cst::Variable;
use mago_word::Word;
use mago_word::concat_word;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::block::BreakContext;
use crate::context::scope::control_action::ControlAction;
use crate::context::scope::loop_scope::LoopScope;
use crate::context::scope::var_has_root;
use crate::error::AnalysisError;
use crate::expression::assignment::PropertyWriteKind;
use crate::expression::assignment::assign_to_expression;
use crate::statement::r#loop;
use crate::utils::expression::get_block_expression_id;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Foreach<'arena> {
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena,
    {
        let iterator = self.expression;
        let is_by_reference = match &self.target {
            ForeachTarget::Value(v) => v.value.is_reference(),
            ForeachTarget::KeyValue(kv) => kv.value.is_reference(),
        };

        let iterator_variable_id = get_block_expression_id(iterator, context, block_context);

        let (always_enters_loop, mut key_type, mut value_type) =
            r#loop::analyze_iterator(context, block_context, artifacts, iterator, iterator_variable_id, self)?;

        if key_type.is_never() {
            key_type = get_mixed();
        }

        if value_type.is_never() {
            value_type = get_mixed();
        }

        let mut loop_block_context = block_context.clone();
        loop_block_context.flags.set_inside_loop(true);
        loop_block_context.break_types.push(BreakContext::Loop(self.span()));
        let mut key_expression_id = None;
        if let Some(key_expression) = self.target.key() {
            key_expression_id = get_block_expression_id(key_expression, context, block_context);

            let assigned = assign_to_expression(
                context,
                &mut loop_block_context,
                artifacts,
                key_expression,
                key_expression_id,
                None,
                Rc::new(key_type),
                false,
                PropertyWriteKind::Direct,
            )?;

            if !assigned {
                context.collector.report_with_code(
                    IssueCode::InvalidForeachKey,
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

        let value_expression = match self.target.value() {
            Expression::UnaryPrefix(UnaryPrefix { operator: UnaryPrefixOperator::Reference(_), operand }) => operand,
            value => value,
        };

        let value_expression_id = get_block_expression_id(value_expression, context, block_context);

        value_type.set_by_reference(is_by_reference);

        if is_by_reference && let Expression::Variable(Variable::Direct(direct_variable)) = value_expression {
            loop_block_context.references_to_external_scope.remove(&Word::from(direct_variable.name));
        }

        let source_value_type = value_type.clone();
        let assigned = assign_to_expression(
            context,
            &mut loop_block_context,
            artifacts,
            value_expression,
            value_expression_id,
            None,
            Rc::new(value_type),
            false,
            if is_by_reference { PropertyWriteKind::Mutation } else { PropertyWriteKind::Direct },
        )?;

        if !assigned {
            context.collector.report_with_code(
                IssueCode::InvalidForeachValue,
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

        if is_by_reference && let Expression::Variable(Variable::Direct(direct_variable)) = value_expression {
            loop_block_context.references_to_external_scope.insert(Word::from(direct_variable.name));
        }

        let foreach_entry = if !is_by_reference
            && matches!(iterator, Expression::Variable(Variable::Direct(_)))
            && self.target.key().is_some_and(|key| matches!(key, Expression::Variable(Variable::Direct(_))))
            && matches!(value_expression, Expression::Variable(Variable::Direct(_)))
            && let (Some(iterator_id), Some(key_id), Some(value_id)) =
                (iterator_variable_id, key_expression_id, value_expression_id)
        {
            let entry_id = concat_word!(iterator_id.as_bytes(), b"[", key_id.as_bytes(), b"]");
            loop_block_context.locals.insert(entry_id, Rc::new(source_value_type.clone()));
            loop_block_context.derived_local_sources.insert(value_id, entry_id);

            Some((iterator_id, key_id, value_id, entry_id, Rc::new(source_value_type)))
        } else {
            None
        };

        let mut loop_scope = LoopScope::new(self.span(), block_context.locals.clone(), None);
        if let Some((_, _, _, entry_id, entry_type)) = &foreach_entry {
            loop_scope.parent_context_variables.insert(*entry_id, Rc::clone(entry_type));
            loop_scope.tracks_assignment_targets = true;
        }

        let (inner_loop_block_context, loop_scope) = r#loop::analyze(
            context,
            self.body.statements(),
            &[],
            vec![],
            loop_scope,
            &mut loop_block_context,
            block_context,
            artifacts,
            false,
            always_enters_loop,
        )?;

        if let Some((iterator_id, key_id, value_id, entry_id, _)) = foreach_entry
            && !loop_scope.final_actions.contains(ControlAction::Break)
            && assignments_only_target_current_entry(
                iterator_id,
                key_id,
                value_id,
                entry_id,
                &loop_scope.assignment_targets,
            )
            && let Some(entry_type) = inner_loop_block_context.locals.get(&entry_id)
            && !entry_type.is_mixed()
            && let Some(iterator_type) = block_context.locals.get(&iterator_id)
            && let Some(refined) = refine_foreach_array_values(iterator_type, entry_type)
        {
            block_context.locals.insert(iterator_id, Rc::new(refined));
        }

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

fn assignments_only_target_current_entry(
    iterator_id: Word,
    key_id: Word,
    value_id: Word,
    entry_id: Word,
    assignment_targets: &mago_word::WordSet,
) -> bool {
    for variable_id in assignment_targets {
        if *variable_id == key_id
            || *variable_id == value_id
            || (*variable_id != entry_id && var_has_root(*variable_id, iterator_id))
        {
            return false;
        }
    }

    true
}

fn refine_foreach_array_values(array_type: &TUnion, value_type: &TUnion) -> Option<TUnion> {
    let mut refined = array_type.clone();

    for atomic in refined.types.to_mut() {
        let TAtomic::Array(array) = atomic else {
            return None;
        };

        if array.is_empty() {
            continue;
        }

        match array {
            TArray::List(list) => {
                if !list.element_type.is_never() {
                    list.element_type = Arc::new(value_type.clone());
                }

                if let Some(known_elements) = list.known_elements.as_mut() {
                    for (_, element_type) in known_elements.values_mut() {
                        *element_type = value_type.clone();
                    }
                }
            }
            TArray::Keyed(keyed) => {
                if let Some(known_items) = keyed.known_items.as_mut() {
                    for (_, item_type) in known_items.values_mut() {
                        *item_type = value_type.clone();
                    }
                }

                if let Some((key_type, _)) = keyed.parameters.as_ref() {
                    keyed.parameters = Some((Arc::clone(key_type), Arc::new(value_type.clone())));
                } else if keyed.known_items.is_none() {
                    keyed.parameters = Some((Arc::new(get_arraykey()), Arc::new(value_type.clone())));
                }
            }
        }
    }

    Some(refined)
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = foreach_basic,
        code = indoc! {"
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
        "}
    }

    test_analysis! {
        name = iterating_over_intersection,
        code = indoc! {r#"
            <?php

            /**
             * @template K
             * @template-covariant V
             */
            interface Traversable
            {
            }

            /**
             * @template K
             * @template-covariant V
             *
             * @extends Traversable<K, V>
             */
            interface IteratorAggregate extends Traversable
            {
                /**
                 * @return Traversable<K, V>
                 */
                public function getIterator(): Traversable;
            }

            class X
            {
            }

            /**
             * @implements IteratorAggregate<int, string>
             */
            class Y extends X implements IteratorAggregate
            {
                /**
                 * @return Traversable<int, string>
                 */
                public function getIterator(): Traversable
                {
                    return $this->getIterator();
                }
            }

            /**
             * @return X&Traversable<int, string>
             */
            function y(): X
            {
                return new Y();
            }

            foreach (y() as $item) {
                echo $item . "\n";
            }
        "#},
        issues = [
            // Traversable: K and V not used in interface body
            IssueCode::UnusedTemplateParameter,
            IssueCode::UnusedTemplateParameter,
        ]
    }
}
