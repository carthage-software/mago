use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_hir::ir::item::statement::constant::Constant;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_constant_declaration(
        &mut self,
        constant: &'source Constant<'source, SymbolId, S, E>,
    ) -> InferenceResult<Constant<'arena, SymbolId, Flow, Type<'arena>>> {
        let attributes = self.infer_attributes(constant.attributes)?;
        let annotation = match constant.annotation {
            Some(annotation) => Some(&*self.arena.alloc(self.infer_item_annotation(annotation)?)),
            None => None,
        };
        let value = self.infer_expression(constant.value)?;

        Ok(Constant {
            span: constant.span,
            annotation,
            attributes,
            version_constraint: self.arena.alloc_slice_copy(constant.version_constraint),
            name: constant.name.copy_into(self.arena),
            value: self.arena.alloc(value),
            flattened: constant.flattened,
        })
    }
}
