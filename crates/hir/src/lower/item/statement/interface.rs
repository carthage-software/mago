use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::item::annotation::generics::TypeParameterDefiningEntity;
use crate::ir::item::statement::interface::Interface;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_interface(
        &mut self,
        interface: &'scratch cst::Interface<'scratch>,
    ) -> &'arena Interface<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&interface.attribute_lists);
        let version_constraint = self.lower_version_constraint(&interface.attribute_lists);
        let name = self.lower_declaration_name(&interface.name);
        let extends = interface.extends.as_ref().map(|extends| self.lower_extends(extends));

        let document = self.phpdoc_resolution.get(interface.span());
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::ClassLike(name));
        let annotation = self.lower_item_annotation(document.as_ref(), Some(name));
        let members = self.lower_members(interface.left_brace.join(interface.right_brace), &interface.members, name);
        self.type_resolution.leave_scope();

        self.arena.alloc(Interface {
            span: interface.span(),
            annotation,
            attributes,
            version_constraint,
            name,
            extends,
            members,
        })
    }
}
