use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::item::member::property::HookedProperty;
use crate::ir::item::member::property::Property;
use crate::ir::item::member::property::PropertyItem;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_plain_property(
        &mut self,
        property: &'scratch cst::PlainProperty<'scratch>,
    ) -> Property<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&property.attribute_lists);
        let modifiers = self.lower_modifiers(&property.modifiers);
        let r#type = property.hint.as_ref().map(|hint| self.lower_type(hint));
        let version_constraint = self.lower_version_constraint(&property.attribute_lists);
        let document = self.phpdoc_resolution.get(property.span());
        let annotation = self.lower_item_annotation(document.as_ref(), None);
        let items = self.arena.alloc_slice_fill_iter(property.items.iter().map(|item| self.lower_property_item(item)));

        Property { span: property.span(), annotation, attributes, version_constraint, modifiers, r#type, items }
    }

    pub(crate) fn lower_hooked_property(
        &mut self,
        property: &'scratch cst::HookedProperty<'scratch>,
    ) -> HookedProperty<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&property.attribute_lists);
        let modifiers = self.lower_modifiers(&property.modifiers);
        let r#type = property.hint.as_ref().map(|hint| self.lower_type(hint));
        let version_constraint = self.lower_version_constraint(&property.attribute_lists);
        let document = self.phpdoc_resolution.get(property.span());
        let annotation = self.lower_item_annotation(document.as_ref(), None);
        let item = self.lower_property_item(&property.item);
        let hooks = self.lower_property_hooks(&property.hook_list);

        HookedProperty {
            span: property.span(),
            annotation,
            attributes,
            version_constraint,
            modifiers,
            r#type,
            item,
            hooks,
        }
    }

    fn lower_property_item(&mut self, item: &'scratch cst::PropertyItem<'scratch>) -> PropertyItem<'arena, (), (), ()> {
        match item {
            cst::PropertyItem::Abstract(item) => PropertyItem {
                span: item.span(),
                variable: self.lower_direct_variable(&item.variable),
                default_value: None,
            },
            cst::PropertyItem::Concrete(item) => PropertyItem {
                span: item.span(),
                variable: self.lower_direct_variable(&item.variable),
                default_value: Some(self.arena.alloc(self.lower_expression(item.value))),
            },
        }
    }
}
