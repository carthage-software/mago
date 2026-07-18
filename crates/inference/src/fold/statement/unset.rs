use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::expression::AccessKind;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::Terminator;
use mago_hir::ir::variable::Variable;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::var::Var;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_unset(
        &mut self,
        span: Span,
        terminator: Option<Terminator>,
        operands_span: Span,
        operands: &'source [Expression<'source, SymbolId, S, E>],
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let mut items = Vec::new_in(self.arena);
        for operand in operands {
            let typed = self.infer_expression(operand)?;
            match typed.kind {
                ExpressionKind::Variable(Variable::Direct(direct)) => {
                    self.environment.unset(Var::new(direct.name));
                }
                ExpressionKind::Access(access) => {
                    if let AccessKind::Array(base, index) = access.kind
                        && let ExpressionKind::Variable(Variable::Direct(direct)) = base.kind
                        && let Some(key) = self.array_key_of(index.meta)
                    {
                        let updated = self.remove_array_key(base.meta, key);
                        self.environment.set(Var::new(direct.name), updated);
                    }
                }
                _ => {}
            }

            items.push(typed);
        }

        Ok(Statement {
            meta: Flow { reachable: self.reachable, exit: ControlFlow::Fallthrough },
            span,
            kind: StatementKind::Unset(Delimited { span: operands_span, items: items.leak() }),
            terminator,
        })
    }
}
