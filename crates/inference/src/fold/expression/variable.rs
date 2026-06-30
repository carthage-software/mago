use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::vec::Vec;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::variable::Variable;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::var::Var;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;
use crate::semantics::literal_string_bytes;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_variable(
        &mut self,
        span: Span,
        variable: &Variable<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let (meta, node) = self.fold_variable(variable)?;

        Ok(Expression { meta, span, kind: ExpressionKind::Variable(node) })
    }

    pub(crate) fn fold_variable(
        &mut self,
        variable: &Variable<'source, SymbolId, S, E>,
    ) -> InferenceResult<(Type<'arena>, Variable<'arena, SymbolId, Flow, Type<'arena>>)> {
        Ok(match variable {
            Variable::Direct(direct) => {
                let direct = direct.copy_into(self.arena);
                let meta = self.environment.get(Var::new(direct.name));

                (meta, Variable::Direct(direct))
            }
            Variable::Indirect(expression) => {
                let expression = self.infer_expression(expression)?;
                let meta = self.dynamic_variable_type(expression.meta);

                (meta, Variable::Indirect(self.arena.alloc(expression)))
            }
            Variable::Nested(inner) => {
                let (name, inner) = self.fold_variable(inner)?;
                let meta = self.dynamic_variable_type(name);

                (meta, Variable::Nested(self.arena.alloc(inner)))
            }
        })
    }

    pub(crate) fn infer_variable_node(
        &mut self,
        variable: &Variable<'source, SymbolId, S, E>,
    ) -> InferenceResult<Variable<'arena, SymbolId, Flow, Type<'arena>>> {
        Ok(self.fold_variable(variable)?.1)
    }

    fn dynamic_variable_type(&self, name_type: Type<'arena>) -> Type<'arena> {
        match self.resolved_variable_name(name_type) {
            Some(name) => self.environment.get(Var::new(name)),
            None => TYPE_MIXED,
        }
    }

    pub(crate) fn resolved_variable_name(&self, name_type: Type<'arena>) -> Option<&'arena [u8]> {
        let value = literal_string_bytes(name_type)?;

        let mut name = Vec::new_in(self.arena);
        name.push(b'$');
        name.extend_from_slice(value);

        Some(name.leak())
    }
}
