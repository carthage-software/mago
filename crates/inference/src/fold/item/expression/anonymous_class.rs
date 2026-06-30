use mago_allocator::Arena;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;
use mago_allocator::vec::Vec;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::item::expression::anonymous_class::AnonymousClass;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_anonymous_class(
        &mut self,
        symbol: SymbolId,
        node: &'source AnonymousClass<'source, SymbolId, S, E>,
    ) -> InferenceResult<(Type<'arena>, AnonymousClass<'arena, SymbolId, Flow, Type<'arena>>)> {
        let (class_name, this_type) = self.class_context(symbol, node.name);

        let attributes = self.infer_attributes(node.attributes)?;
        let annotation = match node.annotation {
            Some(annotation) => Some(&*self.arena.alloc(self.infer_item_annotation(annotation)?)),
            None => None,
        };
        let arguments = match &node.arguments {
            Some(arguments) => Some(self.infer_partial_arguments(arguments)?),
            None => None,
        };

        let outer_class = self.self_class.replace(class_name);
        let mut members = Vec::new_in(self.arena);
        for member in node.members.items {
            members.push(self.infer_member(class_name, this_type, member)?);
        }
        self.self_class = outer_class;

        let atom = self.ty.named_object_atom(class_name);
        let meta = self.ty.union_of(&[atom]);

        let node = AnonymousClass {
            span: node.span,
            modifiers: copy_slice_into(node.modifiers, self.arena),
            name: self.arena.alloc_slice_copy(node.name),
            annotation,
            attributes,
            arguments,
            extends: node.extends.map(|extends| copy_ref_into(extends, self.arena)),
            implements: node.implements.map(|implements| copy_ref_into(implements, self.arena)),
            members: Delimited { span: node.members.span, items: members.leak() },
        };

        Ok((meta, node))
    }
}
