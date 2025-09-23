use std::borrow::Cow;
use std::rc::Rc;

use mago_codex::ttype::TType;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
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
use crate::error::AnalysisError;

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

    if lhs_type.is_null() {
        context.collector.report_with_code(
            IssueCode::RedundantNullCoalesce,
            Issue::help("Redundant null coalesce: left-hand side is always `null`.")
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is always `null`"))
                .with_annotation(
                    Annotation::secondary(binary.rhs.span())
                        .with_message("This right-hand side will always be evaluated"),
                )
                .with_note("The right-hand side of `??` will always be evaluated.")
                .with_help("Consider directly using the right-hand side expression."),
        );

        binary.rhs.analyze(context, block_context, artifacts)?;
        result_type = artifacts.get_expression_type(&binary.rhs).cloned().unwrap_or_else(get_mixed); // Fallback if RHS analysis fails
    } else if !lhs_type.has_nullish() && !lhs_type.possibly_undefined && !lhs_type.possibly_undefined_from_try {
        context.collector.report_with_code(
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

        result_type = combine_union_types(&non_null_lhs_type, &rhs_type, context.codebase, false);
    }

    artifacts.expression_types.insert(get_expression_range(binary), Rc::new(result_type));

    Ok(())
}
