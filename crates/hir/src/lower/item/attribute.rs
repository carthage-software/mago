use mago_allocator::Arena;
use mago_allocator::vec::Vec;

use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::item::attribute::Attribute;
use crate::lower::Lowering;
use crate::lower::resolution::namespace::NameResolutionKind;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_attribute_lists(
        &mut self,
        attribute_lists: &'scratch cst::Sequence<'scratch, cst::AttributeList<'scratch>>,
    ) -> &'arena [Attribute<'arena, (), (), ()>] {
        let mut attributes = Vec::new_in(self.arena);
        for attribute_list in attribute_lists.iter() {
            for attribute in attribute_list.attributes.iter() {
                let lowered = self.lower_attribute(attribute);

                attributes.push(lowered);
            }
        }

        attributes.leak()
    }

    fn lower_attribute(&mut self, attribute: &'scratch cst::Attribute<'scratch>) -> Attribute<'arena, (), (), ()> {
        Attribute {
            span: attribute.span(),
            class: self.lower_identifier(&attribute.name, Some(NameResolutionKind::Default)),
            arguments: attribute
                .argument_list
                .as_ref()
                .map(|argument_list| self.lower_partial_argument_list(argument_list)),
        }
    }
}
