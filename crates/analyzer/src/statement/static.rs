use mago_allocator::Arena;
use std::mem::replace;
use std::rc::Rc;
use std::sync::Arc;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Statement;
use mago_syntax::cst::Static;
use mago_syntax::walker::MutWalker;
use mago_syntax::walker::walk_statement_mut;
use mago_word::Word;
use mago_word::WordMap;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::block::ReferenceConstraint;
use crate::context::block::ReferenceConstraintSource;
use crate::error::AnalysisError;
use crate::statement::analyze_statements;
use crate::utils::docblock::check_docblock_type_incompatibility;
use crate::utils::docblock::get_type_from_var_docblock;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Static<'arena> {
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena,
    {
        if block_context.scope.is_pure() {
            context.collector.report_with_code(
                IssueCode::ImpureStaticVariable,
                Issue::error(
                    "Cannot declare `static` variables inside a pure function or method."
                )
                .with_annotation(
                    Annotation::primary(self.span()).with_message("`static` variable declared here.")
                )
                .with_note(
                    "Static variables maintain state across function calls, which violates the pure guarantee."
                )
                .with_help(
                    "Remove the `static` declaration or remove the `@pure` annotation from the enclosing function/method."
                ),
            );
        }

        for item in &self.items {
            let variable = item.variable();
            let initial_value = item.value();

            let mut inferred_type = None;
            if let Some(initial_value) = initial_value {
                let was_inside_general_use = block_context.flags.inside_general_use();
                block_context.flags.set_inside_general_use(true);
                initial_value.analyze(context, block_context, artifacts)?;
                block_context.flags.set_inside_general_use(was_inside_general_use);

                inferred_type = artifacts.get_rc_expression_type(initial_value).cloned();
            }

            let variable_span = variable.span();

            let docblock_type = get_type_from_var_docblock(
                context,
                block_context,
                artifacts,
                Some(variable.name),
                self.items.len() == 1,
            );

            let variable_name_atom = Word::from(variable.name);
            let variable_type = match (inferred_type, docblock_type) {
                (Some(inferred_type), Some((docblock_type, docblock_type_span))) => {
                    let docblock_type = Rc::new(docblock_type);
                    block_context.by_reference_constraints.insert(
                        variable_name_atom,
                        ReferenceConstraint::new(
                            docblock_type_span,
                            ReferenceConstraintSource::Static,
                            Some(Rc::clone(&docblock_type)),
                        ),
                    );

                    check_docblock_type_incompatibility(
                        context,
                        Some(variable.name),
                        variable_span,
                        &inferred_type,
                        &docblock_type,
                        docblock_type_span,
                        initial_value,
                    );

                    docblock_type
                }
                (None, Some((docblock_type, docblock_type_span))) => {
                    let docblock_type = Rc::new(docblock_type);
                    block_context.by_reference_constraints.insert(
                        variable_name_atom,
                        ReferenceConstraint::new(
                            docblock_type_span,
                            ReferenceConstraintSource::Static,
                            Some(Rc::clone(&docblock_type)),
                        ),
                    );

                    docblock_type
                }
                (inferred_type, None) => artifacts
                    .static_local_types
                    .as_ref()
                    .and_then(|types| types.get(&variable_name_atom))
                    .map(|ty| Rc::new(ty.clone()))
                    .or(inferred_type)
                    .unwrap_or_else(|| Rc::new(get_mixed())),
            };

            block_context.locals.insert(variable_name_atom, variable_type);
            block_context.assigned_variable_ids.insert(variable_name_atom, item.span().start.offset);
            block_context.static_locals.insert(variable_name_atom);
        }

        Ok(())
    }
}

pub(super) fn infer_static_local_types<'ctx, 'arena, A>(
    context: &mut Context<'ctx, 'arena, A>,
    block_context: &BlockContext<'ctx>,
    artifacts: &AnalysisArtifacts,
    statements: &[Statement<'arena>],
) -> Result<Option<WordMap<TUnion>>, AnalysisError>
where
    A: Arena,
{
    let mut finder = StaticLocalFinder::default();
    for statement in statements {
        finder.walk_statement(statement, &mut ());
    }

    if !finder.0 {
        return Ok(None);
    }

    let scope = context.scope.clone();
    let mut static_local_types = WordMap::default();
    let mut previous_types = WordMap::default();
    for _ in 0..8 {
        let mut inferred_context = block_context.clone();
        let mut inferred_artifacts = artifacts.clone();
        inferred_artifacts.static_local_types = Some(static_local_types.clone());

        let (result, _) = context
            .record(|context| analyze_statements(statements, context, &mut inferred_context, &mut inferred_artifacts));
        context.scope.clone_from(&scope);
        result?;

        let mut inferred_types = inferred_artifacts.static_local_types.unwrap_or_default();
        if inferred_types == static_local_types {
            return Ok(Some(inferred_types));
        }

        for (variable, inferred_type) in &mut inferred_types {
            if let Some(previous) = static_local_types.get(variable)
                && previous != inferred_type
            {
                widen_static_local_type(inferred_type, previous);
            }
        }

        if inferred_types == static_local_types {
            return Ok(Some(inferred_types));
        }

        previous_types = replace(&mut static_local_types, inferred_types);
    }

    for (variable, variable_type) in &mut static_local_types {
        if previous_types.get(variable) != Some(variable_type) {
            *variable_type = get_mixed();
        }
    }

    Ok(Some(static_local_types))
}

fn widen_static_local_type(inferred: &mut TUnion, previous: &TUnion) {
    if inferred == previous {
        return;
    }

    match (inferred.types.to_mut().as_mut_slice(), previous.get_single_array()) {
        ([TAtomic::Array(TArray::Keyed(inferred))], Some(TArray::Keyed(previous))) => {
            if let (Some(items), Some(previous_items)) = (&mut inferred.known_items, &previous.known_items) {
                for (key, (_, item_type)) in items {
                    if let Some((_, previous_type)) = previous_items.get(key) {
                        widen_static_local_type(item_type, previous_type);
                    }
                }
            }

            if let (Some((key, value)), Some((previous_key, previous_value))) =
                (&mut inferred.parameters, &previous.parameters)
            {
                widen_static_local_type(Arc::make_mut(key), previous_key);
                widen_static_local_type(Arc::make_mut(value), previous_value);
            }
        }
        ([TAtomic::Array(TArray::List(inferred))], Some(TArray::List(previous))) => {
            if let (Some(items), Some(previous_items)) = (&mut inferred.known_elements, &previous.known_elements) {
                for (key, (_, item_type)) in items {
                    if let Some((_, previous_type)) = previous_items.get(key) {
                        widen_static_local_type(item_type, previous_type);
                    }
                }
            }

            widen_static_local_type(Arc::make_mut(&mut inferred.element_type), &previous.element_type);
        }
        _ => inferred.widen_literals(),
    }
}

#[derive(Default)]
struct StaticLocalFinder(bool);

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for StaticLocalFinder {
    fn walk_static(&mut self, _: &'ast Static<'arena>, _: &mut ()) {
        self.0 = true;
    }

    fn walk_statement(&mut self, statement: &'ast Statement<'arena>, context: &mut ()) {
        if !self.0 && !statement.is_declaration() {
            walk_statement_mut(self, statement, context);
        }
    }

    fn walk_expression(&mut self, _: &'ast Expression<'arena>, _: &mut ()) {}
}
