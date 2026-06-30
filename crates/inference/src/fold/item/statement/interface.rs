use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::vec::Vec;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::item::statement::interface::Interface;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_interface(
        &mut self,
        symbol: SymbolId,
        node: &'source Interface<'source, SymbolId, S, E>,
    ) -> InferenceResult<Interface<'arena, SymbolId, Flow, Type<'arena>>> {
        let (class_name, this_type) = self.class_context(symbol, node.name.value);

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

        Ok(Interface {
            span: node.span,
            annotation,
            attributes,
            version_constraint: self.arena.alloc_slice_copy(node.version_constraint),
            name: node.name.copy_into(self.arena),
            extends: node.extends.map(|extends| copy_ref_into(extends, self.arena)),
            members: Delimited { span: node.members.span, items: members.leak() },
        })
    }
}
