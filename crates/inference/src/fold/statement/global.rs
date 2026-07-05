use mago_allocator::Arena;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::vec::Vec;
use mago_hir::ir::statement::GlobalItem;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::Terminator;
use mago_hir::ir::variable::Variable;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_MIXED;
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
    pub(crate) fn infer_global(
        &mut self,
        span: Span,
        terminator: Option<Terminator>,
        items: &'source [GlobalItem<'source, SymbolId, S, E>],
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let mut typed_items = Vec::new_in(self.arena);
        for item in items {
            let ty = self.lowered_annotation(item.type_annotation).unwrap_or(TYPE_MIXED);
            if let Variable::Direct(direct) = &item.variable {
                self.environment.set(Var::new(self.arena.alloc_slice_copy(direct.name)), ty);
            }

            let variable = self.infer_variable_node(&item.variable)?;
            let type_annotation = item.type_annotation.map(|annotation| copy_ref_into(annotation, self.arena));
            typed_items.push(GlobalItem { span: item.span, variable, type_annotation });
        }

        Ok(Statement {
            meta: Flow { reachable: self.reachable, exit: ControlFlow::Fallthrough },
            span,
            kind: StatementKind::Global(typed_items.leak()),
            terminator,
        })
    }
}
