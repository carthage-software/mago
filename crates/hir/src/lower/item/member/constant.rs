use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::item::member::constant::ClassLikeConstant;
use crate::ir::item::member::constant::ClassLikeConstantItem;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_class_like_constant(
        &mut self,
        constant: &'scratch cst::ClassLikeConstant<'scratch>,
    ) -> ClassLikeConstant<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&constant.attribute_lists);
        let version_constraint = self.lower_version_constraint(&constant.attribute_lists);
        let modifiers = self.lower_modifiers(&constant.modifiers);
        let r#type = constant.hint.as_ref().map(|hint| self.lower_type(hint));
        let document = self.phpdoc_resolution.get(constant.span());
        let annotation = self.lower_item_annotation(document.as_ref(), None);
        let items = self
            .arena
            .alloc_slice_fill_iter(constant.items.iter().map(|item| self.lower_class_like_constant_item(item)));

        ClassLikeConstant {
            span: constant.span(),
            annotation,
            attributes,
            version_constraint,
            modifiers,
            r#type,
            items,
        }
    }

    fn lower_class_like_constant_item(
        &mut self,
        item: &'scratch cst::ClassLikeConstantItem<'scratch>,
    ) -> ClassLikeConstantItem<'arena, (), (), ()> {
        ClassLikeConstantItem {
            span: item.span(),
            name: self.lower_name(&item.name),
            value: self.arena.alloc(self.lower_expression(item.value)),
        }
    }
}
