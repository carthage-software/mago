use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::conditional::analyze_conditional;

/// Analyzes the Elvis operator (`?:`).
pub fn analyze_elvis_operation<'ctx, 'arena>(
    binary: &Binary<'arena>,
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    analyze_conditional(context, block_context, artifacts, binary.lhs, None, binary.rhs, binary.span())
}
