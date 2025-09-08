use ahash::HashSet;
use indexmap::IndexMap;

use mago_algebra::clause::Clause;
use mago_algebra::find_satisfying_assignments;
use mago_algebra::negate_formula;
use mago_algebra::saturate_clauses;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::block::BreakContext;
use crate::context::scope::loop_scope::LoopScope;
use crate::error::AnalysisError;
use crate::formula::get_formula;
use crate::formula::remove_clauses_with_mixed_variables;
use crate::reconciler::reconcile_keyed_types;
use crate::statement::r#loop;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for DoWhile<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let mut loop_block_context = block_context.clone();
        loop_block_context.break_types.push(BreakContext::Loop);
        loop_block_context.inside_loop = true;

        let loop_scope = LoopScope::new(self.span(), block_context.locals.clone(), None);

        let mut mixed_variable_ids = vec![];
        for (variable_id, variable_type) in &loop_scope.parent_context_variables {
            if variable_type.is_mixed() {
                mixed_variable_ids.push(variable_id);
            }
        }

        let mut while_clauses = get_formula(
            self.condition.span(),
            self.condition.span(),
            self.condition,
            context.get_assertion_context_from_block(block_context),
            artifacts,
        )
        .unwrap_or_else(|| {
            context.collector.report_with_code(
                IssueCode::ConditionIsTooComplex,
                Issue::warning("Loop condition is too complex for precise type analysis.")
                    .with_annotation(
                        Annotation::primary(self.condition.span())
                            .with_message("This `do-while` loop condition is too complex for the analyzer to fully understand"),
                    )
                    .with_annotation(
                        Annotation::secondary(self.statement.span())
                            .with_message("Type inference within the loop statement(s) may be inaccurate as a result"),
                    )
                    .with_note(
                        "To prevent performance issues, the analyzer limits the number of logical paths it explores for a single condition."
                    )
                    .with_note(
                        "Because this limit was exceeded, type assertions from the condition will not be applied, which can affect variable types on subsequent loop iterations."
                    )
                    .with_help(
                        "Consider refactoring this condition into a simpler expression or breaking it down into intermediate boolean variables before the loop.",
                    ),
            );

            vec![]
        });

        while_clauses = remove_clauses_with_mixed_variables(while_clauses, mixed_variable_ids, self.condition.span());
        if while_clauses.is_empty() {
            while_clauses.push(Clause::new(
                IndexMap::new(),
                self.condition.span(),
                self.condition.span(),
                Some(true),
                None,
                None,
            ));
        }

        let span = self.span();
        let previous_loop_bounds = loop_block_context.loop_bounds;
        loop_block_context.loop_bounds = (span.start.offset, span.end.offset);

        let (mut inner_loop_block_context, loop_scope) = r#loop::analyze(
            context,
            std::slice::from_ref(self.statement),
            vec![],
            r#loop::get_and_expressions(self.condition),
            loop_scope,
            &mut loop_block_context,
            block_context,
            artifacts,
            true,
            true,
        )?;

        loop_block_context.loop_bounds = previous_loop_bounds;

        let clauses_to_simplify = {
            let mut c = block_context.clauses.iter().map(|v| (**v).clone()).collect::<Vec<_>>();
            c.extend(negate_formula(while_clauses).unwrap_or_default());
            c
        };

        let (negated_while_types, _) = find_satisfying_assignments(
            saturate_clauses(&clauses_to_simplify).as_slice(),
            None,
            &mut HashSet::default(),
        );

        if !negated_while_types.is_empty() {
            reconcile_keyed_types(
                context,
                &negated_while_types,
                IndexMap::new(),
                &mut inner_loop_block_context,
                &mut HashSet::default(),
                &HashSet::default(),
                &self.condition.span(),
                true,
                false,
            );
        }

        let infinite_loop = artifacts.get_expression_type(self.condition).is_some_and(|c| c.is_always_truthy());

        r#loop::inherit_loop_block_context(
            context,
            block_context,
            loop_block_context,
            inner_loop_block_context,
            loop_scope,
            /* always_enters_loop = */ true,
            infinite_loop,
        );

        Ok(())
    }
}
