use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::vec::Vec;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::item::attribute::Attribute;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_attributes(
        &mut self,
        attributes: &[Attribute<'source, SymbolId, S, E>],
    ) -> InferenceResult<&'arena [Attribute<'arena, SymbolId, Flow, Type<'arena>>]> {
        let mut items = Vec::new_in(self.arena);
        for attribute in attributes {
            items.push(self.infer_attribute(attribute)?);
        }

        Ok(items.leak())
    }

    fn infer_attribute(
        &mut self,
        attribute: &Attribute<'source, SymbolId, S, E>,
    ) -> InferenceResult<Attribute<'arena, SymbolId, Flow, Type<'arena>>> {
        let arguments = match attribute.arguments.as_ref() {
            Some(arguments) => {
                let mut items = Vec::new_in(self.arena);
                for argument in arguments.items {
                    items.push(self.infer_partial_argument(argument)?);
                }

                Some(Delimited { span: arguments.span, items: items.leak() })
            }
            None => None,
        };

        Ok(Attribute { span: attribute.span, class: attribute.class.copy_into(self.arena), arguments })
    }
}
