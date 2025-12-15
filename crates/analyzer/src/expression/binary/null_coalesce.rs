use std::borrow::Cow;
use std::rc::Rc;

use mago_algebra::find_satisfying_assignments;
use mago_codex::ttype::TType;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
use mago_fixer::SafetyClassification;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::artifacts::get_expression_range;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::scope::if_scope::IfScope;
use crate::error::AnalysisError;
use crate::formula::get_formula;
use crate::reconciler::reconcile_keyed_types;
use crate::utils::conditional;
use crate::utils::misc::unwrap_expression;

/// Analyzes the null coalescing operator (`??`).
///
/// The result type is determined as follows:
/// - If the left-hand side (LHS) is definitely `null`, the result type is the type of the right-hand side (RHS).
///   A hint is issued about the LHS always being `null`.
/// - If the LHS is definitely not `null`, the result type is the type of the LHS. The RHS is still analyzed
///   for potential errors but does not contribute to the result type. A hint is issued about the RHS being redundant.
/// - If the LHS is nullable (can be `null` or other types), the result type is the union of the
///   non-null parts of the LHS and the type of the RHS.
/// - If the LHS type is unknown (`mixed`), the result type is `mixed`.
pub fn analyze_null_coalesce_operation<'ctx, 'arena>(
    binary: &Binary<'arena>,
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    let was_inside_isset = block_context.inside_isset;
    block_context.inside_isset = true;
    binary.lhs.analyze(context, block_context, artifacts)?;
    block_context.inside_isset = was_inside_isset;

    let lhs_type_option = artifacts.get_rc_expression_type(&binary.lhs);

    let Some(lhs_type) = lhs_type_option else {
        binary.rhs.analyze(context, block_context, artifacts)?;

        artifacts.set_expression_type(binary, get_mixed());

        return Ok(());
    };

    let result_type: TUnion;
    let mut rhs_is_never = false;

    let is_static_var = matches!(
        unwrap_expression(binary.lhs),
        Expression::Variable(Variable::Direct(var)) if block_context.static_locals.contains(&mago_atom::atom(var.name))
    );

    if lhs_type.is_null() && !is_static_var {
        context.collector.propose_with_code(
            IssueCode::RedundantNullCoalesce,
            Issue::help("Redundant null coalesce: left-hand side is always `null`.")
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is always `null`"))
                .with_annotation(
                    Annotation::secondary(binary.rhs.span())
                        .with_message("This right-hand side will always be evaluated"),
                )
                .with_note("The right-hand side of `??` will always be evaluated.")
                .with_help("Consider directly using the right-hand side expression."),
            |plan| {
                plan.delete(
                    binary.lhs.span().join(binary.operator.span()).to_range(),
                    SafetyClassification::PotentiallyUnsafe,
                );
            },
        );

        binary.rhs.analyze(context, block_context, artifacts)?;
        result_type = artifacts.get_expression_type(&binary.rhs).cloned().unwrap_or_else(get_mixed); // Fallback if RHS analysis fails
    } else if !lhs_type.has_nullish() && !lhs_type.possibly_undefined() && !lhs_type.possibly_undefined_from_try() {
        context.collector.propose_with_code(
            IssueCode::RedundantNullCoalesce,
            Issue::help(
                "Redundant null coalesce: left-hand side can never be `null` or undefined."
            )
            .with_annotation(Annotation::primary(binary.lhs.span()).with_message(format!(
                "This expression (type `{}`) is never `null` or undefined",
                lhs_type.get_id()
            )))
            .with_annotation(
                Annotation::secondary(binary.rhs.span()).with_message("This right-hand side will never be evaluated"),
            )
            .with_note(
                "The null coalesce operator `??` only evaluates the right-hand side if the left-hand side is `null` or not set.",
            )
            .with_help("Consider removing the `??` operator and the right-hand side expression."),
            |plan| {
                plan.delete(
                    binary.operator.span().join(binary.rhs.span()).to_range(),
                    SafetyClassification::PotentiallyUnsafe,
                );
            },
        );

        result_type = (**lhs_type).clone();
        binary.rhs.analyze(context, block_context, artifacts)?;
    } else {
        let non_null_lhs_type = lhs_type.to_non_nullable();
        let has_returned = block_context.has_returned;
        block_context.has_returned = false;
        binary.rhs.analyze(context, block_context, artifacts)?;
        block_context.has_returned &= has_returned;

        let rhs_type =
            artifacts.get_expression_type(&binary.rhs).map(Cow::Borrowed).unwrap_or_else(|| Cow::Owned(get_mixed()));

        rhs_is_never = rhs_type.is_never();

        result_type = combine_union_types(&non_null_lhs_type, &rhs_type, context.codebase, false);
    }

    artifacts.expression_types.insert(get_expression_range(binary), Rc::new(result_type));

    if rhs_is_never {
        let assertion_context = context.get_assertion_context_from_block(block_context);
        let if_clauses =
            get_formula(binary.lhs.span(), binary.lhs.span(), binary.lhs, assertion_context, artifacts).unwrap();

        let conditional_context_clauses = if_clauses.clone().into_iter().map(Rc::new).collect::<Vec<_>>();

        let mut if_scope = IfScope::new();
        let (if_conditional_scope, _) =
            conditional::analyze(context, block_context.clone(), artifacts, &mut if_scope, binary.lhs, false)?;

        let mut conditionally_referenced_variable_ids = if_conditional_scope.conditionally_referenced_variable_ids;

        let (reconcilable_if_types, active_if_types) = find_satisfying_assignments(
            conditional_context_clauses.into_iter().map(|rc| (*rc).clone()).collect::<Vec<_>>().as_slice(),
            Some(binary.lhs.span()),
            &mut conditionally_referenced_variable_ids,
        );

        let mut changed_variable_ids = mago_atom::AtomSet::default();

        reconcile_keyed_types(
            context,
            &reconcilable_if_types,
            active_if_types,
            block_context,
            &mut changed_variable_ids,
            &conditionally_referenced_variable_ids,
            &binary.lhs.span(),
            false,
            false,
        );
    }

    Ok(())
}
