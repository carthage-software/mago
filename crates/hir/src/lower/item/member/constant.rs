use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::item::member::constant::ClassLikeConstant;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    /// Lowers a class-like constant declaration into one node per declared
    /// constant, in source order, so `const A = 1, B = 2;` yields two nodes.
    pub(crate) fn lower_class_like_constant(
        &mut self,
        constant: &'scratch cst::ClassLikeConstant<'scratch>,
    ) -> Vec<ClassLikeConstant<'arena, (), (), ()>> {
        let attributes = self.lower_attribute_lists(&constant.attribute_lists);
        let version_constraint = self.lower_version_constraint(&constant.attribute_lists);
        let modifiers = self.lower_modifiers(&constant.modifiers);
        let r#type = constant.hint.as_ref().map(|hint| self.lower_type(hint));
        let document = self.phpdoc_resolution.get(constant.span());
        let annotation = self.lower_item_annotation(document.as_ref(), None);
        let flattened = constant.items.len() > 1;

        constant
            .items
            .iter()
            .map(|item| {
                let name = self.lower_name(&item.name);
                let value = self.arena.alloc(self.lower_expression(item.value));

                ClassLikeConstant {
                    span: item.span(),
                    annotation,
                    attributes,
                    version_constraint,
                    modifiers,
                    r#type,
                    name,
                    value,
                    flattened,
                }
            })
            .collect()
    }
}
