use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::vec::Vec;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::StaticItem;
use mago_hir::ir::statement::Terminator;
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
    pub(crate) fn infer_static(
        &mut self,
        span: Span,
        terminator: Option<Terminator>,
        items: &'source [StaticItem<'source, SymbolId, S, E>],
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let mut typed_items = Vec::new_in(self.arena);
        let mut diverges = false;
        for item in items {
            let value = match item.value {
                Some(value) => {
                    let typed = self.infer_expression(value)?;
                    diverges |= typed.meta.is_never();

                    Some(&*self.arena.alloc(typed))
                }
                None => None,
            };

            let ty = self
                .lowered_annotation(item.type_annotation)
                .or_else(|| value.map(|value| value.meta))
                .unwrap_or(TYPE_MIXED);
            self.environment.set(Var::new(self.arena.alloc_slice_copy(item.variable.name)), ty);

            let type_annotation = item.type_annotation.map(|annotation| copy_ref_into(annotation, self.arena));
            typed_items.push(StaticItem {
                span: item.span,
                variable: item.variable.copy_into(self.arena),
                type_annotation,
                value,
            });
        }

        let exit = if diverges { ControlFlow::Diverge } else { ControlFlow::Fallthrough };

        Ok(Statement {
            meta: Flow { reachable: self.reachable, exit },
            span,
            kind: StatementKind::Static(typed_items.leak()),
            terminator,
        })
    }
}
