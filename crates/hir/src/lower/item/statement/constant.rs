use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::item::statement::constant::Constant;
use crate::ir::item::statement::constant::ConstantItem;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_constant(
        &mut self,
        constant: &'scratch cst::Constant<'scratch>,
    ) -> &'arena Constant<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&constant.attribute_lists);
        let version_constraint = self.lower_version_constraint(&constant.attribute_lists);
        let document = self.phpdoc_resolution.get(constant.span());
        let annotation = self.lower_item_annotation(document.as_ref(), None);
        let items = self.arena.alloc_slice_fill_iter(constant.items.iter().map(|item| ConstantItem {
            span: item.span(),
            name: self.lower_declaration_name(&item.name),
            value: self.arena.alloc(self.lower_expression(item.value)),
        }));

        self.arena.alloc(Constant { span: constant.span(), annotation, attributes, version_constraint, items })
    }
}
