use bumpalo::collections::Vec;

use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::attribute::Attribute;
use crate::lower::Lowering;
use crate::lower::resolution::namespace::NameResolutionKind;

impl<'arena> Lowering<'_, 'arena> {
    pub(crate) fn lower_attribute_lists(
        &mut self,
        attribute_lists: &'arena cst::Sequence<'arena, cst::AttributeList<'arena>>,
    ) -> &'arena [Attribute<'arena, (), (), ()>] {
        let mut attributes = Vec::new_in(self.arena);
        for attribute_list in attribute_lists.iter() {
            for attribute in attribute_list.attributes.iter() {
                let lowered = self.lower_attribute(attribute);

                attributes.push(lowered);
            }
        }

        attributes.into_bump_slice()
    }

    fn lower_attribute(&mut self, attribute: &'arena cst::Attribute<'arena>) -> Attribute<'arena, (), (), ()> {
        Attribute {
            span: attribute.span(),
            class: self.lower_identifier(&attribute.name, Some(NameResolutionKind::Default)),
            arguments: match &attribute.argument_list {
                Some(argument_list) => self.lower_argument_list(argument_list),
                None => &[],
            },
        }
    }
}
