use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::vec::Vec;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::statement::Declare;
use mago_hir::ir::statement::DeclareItem;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_declare(
        &mut self,
        span: Span,
        declare: &'source Declare<'source, SymbolId, S, E>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let reachable = self.reachable;
        let mut items = Vec::new_in(self.arena);
        for item in declare.items.items {
            let value = match item.value {
                Some(value) => Some(&*self.arena.alloc(self.infer_expression(value)?)),
                None => None,
            };

            items.push(DeclareItem { span: item.span, name: item.name.copy_into(self.arena), value });
        }

        let statement = self.infer_statement(declare.statement)?;
        let exit = statement.meta.exit;

        let node = Declare {
            span: declare.span,
            items: Delimited { span: declare.items.span, items: items.leak() },
            statement: self.arena.alloc(statement),
        };

        Ok(Statement { meta: Flow { reachable, exit }, span, kind: StatementKind::Declare(self.arena.alloc(node)) })
    }
}
