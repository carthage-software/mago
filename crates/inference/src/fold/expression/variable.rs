use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::variable::Variable;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::var::Var;
use mago_span::Span;

use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_variable(
        &mut self,
        span: Span,
        variable: &'source Variable<'source, SymbolId, S, E>,
    ) -> Expression<'arena, SymbolId, Flow, Type<'arena>> {
        let (meta, kind) = match variable {
            Variable::Direct(direct) => {
                let direct = direct.copy_into(self.arena);
                let meta = self.environment.get(&Var::new(direct.name)).copied().unwrap_or(TYPE_MIXED);

                (meta, ExpressionKind::Variable(Variable::Direct(direct)))
            }
            Variable::Indirect(expression) => {
                let expression = self.infer_expression(expression);

                (TYPE_MIXED, ExpressionKind::Variable(Variable::Indirect(self.arena.alloc(expression))))
            }
            Variable::Nested(inner) => {
                let inner = self.infer_variable_node(inner);

                (TYPE_MIXED, ExpressionKind::Variable(Variable::Nested(self.arena.alloc(inner))))
            }
        };

        Expression { meta, span, kind }
    }

    fn infer_variable_node(
        &mut self,
        variable: &'source Variable<'source, SymbolId, S, E>,
    ) -> Variable<'arena, SymbolId, Flow, Type<'arena>> {
        match variable {
            Variable::Direct(direct) => Variable::Direct(direct.copy_into(self.arena)),
            Variable::Indirect(expression) => {
                let expression = self.infer_expression(expression);

                Variable::Indirect(self.arena.alloc(expression))
            }
            Variable::Nested(inner) => {
                let inner = self.infer_variable_node(inner);

                Variable::Nested(self.arena.alloc(inner))
            }
        }
    }
}
