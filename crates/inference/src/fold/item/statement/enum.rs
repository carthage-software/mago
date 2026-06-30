use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::vec::Vec;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::item::statement::r#enum::Enum;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::Symbol;
use mago_oracle::ty::Type;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_enum(
        &mut self,
        symbol: SymbolId,
        node: &'source Enum<'source, SymbolId, S, E>,
    ) -> InferenceResult<Enum<'arena, SymbolId, Flow, Type<'arena>>> {
        let class_name = match self.symbols.get_class_like(symbol) {
            Some(symbol) => symbol.path().as_bytes(),
            None => self.arena.alloc_slice_copy(node.name.value),
        };

        let this_atom = self.ty.enum_any(class_name);
        let this_type = self.ty.union_of(&[this_atom]);

        let attributes = self.infer_attributes(node.attributes)?;
        let annotation = match node.annotation {
            Some(annotation) => Some(&*self.arena.alloc(self.infer_item_annotation(annotation)?)),
            None => None,
        };

        let outer_class = self.self_class.replace(class_name);
        let mut members = Vec::new_in(self.arena);
        for member in node.members.items {
            members.push(self.infer_member(class_name, this_type, member)?);
        }

        self.self_class = outer_class;

        Ok(Enum {
            span: node.span,
            annotation,
            attributes,
            version_constraint: self.arena.alloc_slice_copy(node.version_constraint),
            name: node.name.copy_into(self.arena),
            backing_type: node.backing_type.map(|backing_type| backing_type.copy_into(self.arena)),
            implements: node.implements.map(|implements| copy_ref_into(implements, self.arena)),
            members: Delimited { span: node.members.span, items: members.leak() },
        })
    }
}
