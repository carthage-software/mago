use mago_allocator::Arena;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::literal::LiteralKind;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::Terminator;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

/// A `break`/`continue` operand inferred: the static level (`None` when the
/// operand is not a literal positive integer — a fatal error) and the typed node.
type LoopLevel<'arena> = (Option<u64>, Option<&'arena Expression<'arena, SymbolId, Flow, Type<'arena>>>);

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_break(
        &mut self,
        span: Span,
        terminator: Option<Terminator>,
        level: Option<&'source Expression<'source, SymbolId, S, E>>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let reachable = self.reachable;
        let (depth, level) = self.infer_loop_level(level)?;
        let exit = match depth {
            Some(depth) => {
                if reachable {
                    self.record_loop_exit(depth, true);
                }

                ControlFlow::Break(depth)
            }
            None => ControlFlow::Diverge,
        };

        Ok(Statement { meta: Flow { reachable, exit }, span, kind: StatementKind::Break(level), terminator })
    }

    /// Infers a `break`/`continue` level operand. PHP requires it to be a literal
    /// positive integer *token*; anything else (a variable, an expression like
    /// `1 + 1`, or `0`) is a fatal error, reported as `None` so the caller
    /// diverges. The operand is still inferred so the typed tree is complete.
    pub(crate) fn infer_loop_level(
        &mut self,
        level: Option<&'source Expression<'source, SymbolId, S, E>>,
    ) -> InferenceResult<LoopLevel<'arena>> {
        match level {
            Some(operand) => {
                let depth = literal_level(operand);
                let operand = self.infer_expression(operand)?;

                Ok((depth, Some(self.arena.alloc(operand))))
            }
            None => Ok((Some(1), None)),
        }
    }
}

/// The positive-integer level a `break`/`continue` operand denotes, only when it
/// is a literal integer token — not merely an expression that *types* as one.
fn literal_level<I, S, E>(expression: &Expression<'_, I, S, E>) -> Option<u64> {
    let ExpressionKind::Literal(literal) = &expression.kind else {
        return None;
    };
    let LiteralKind::Integer(integer) = literal.kind else {
        return None;
    };

    match integer.value {
        Some(value) if value >= 1 => Some(value),
        _ => None,
    }
}
