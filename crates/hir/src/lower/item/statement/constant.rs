use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::item::statement::constant::Constant;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    /// Lowers a global constant declaration into one node per declared
    /// constant, in source order, so `const A = 1, B = 2;` yields two nodes.
    pub(crate) fn lower_constant(
        &mut self,
        constant: &'scratch cst::Constant<'scratch>,
    ) -> Vec<Constant<'arena, (), (), ()>> {
        let attributes = self.lower_attribute_lists(&constant.attribute_lists);
        let version_constraint = self.lower_version_constraint(&constant.attribute_lists);
        let document = self.phpdoc_resolution.get(constant.span());
        let annotation = self.lower_item_annotation(document.as_ref(), None);

        constant
            .items
            .iter()
            .map(|item| {
                let name = self.lower_declaration_name(&item.name);
                let value = self.arena.alloc(self.lower_expression(item.value));

                Constant { span: item.span(), annotation, attributes, version_constraint, name, value }
            })
            .collect()
    }
}
