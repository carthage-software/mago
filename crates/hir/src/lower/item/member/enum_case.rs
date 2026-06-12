use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::item::member::enum_case::EnumCase;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_enum_case(
        &mut self,
        enum_case: &'scratch cst::EnumCase<'scratch>,
    ) -> EnumCase<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&enum_case.attribute_lists);
        let version_constraint = self.lower_version_constraint(&enum_case.attribute_lists);
        let document = self.phpdoc_resolution.get(enum_case.span());
        let annotation = self.lower_item_annotation(document.as_ref(), None);
        let span = enum_case.span();

        match &enum_case.item {
            cst::EnumCaseItem::Unit(unit) => EnumCase {
                span,
                annotation,
                attributes,
                version_constraint,
                name: self.lower_name(&unit.name),
                value: None,
            },
            cst::EnumCaseItem::Backed(backed) => EnumCase {
                span,
                annotation,
                attributes,
                version_constraint,
                name: self.lower_name(&backed.name),
                value: Some(self.arena.alloc(self.lower_expression(backed.value))),
            },
        }
    }
}
