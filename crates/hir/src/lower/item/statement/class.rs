use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::item::annotation::generics::TypeParameterDefiningEntity;
use crate::ir::item::statement::class::Class;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_class(&mut self, class: &'scratch cst::Class<'scratch>) -> &'arena Class<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&class.attribute_lists);
        let version_constraint = self.lower_version_constraint(&class.attribute_lists);
        let modifiers = self.lower_modifiers(&class.modifiers);
        let name = self.lower_declaration_name(&class.name);
        let extends = class.extends.as_ref().map(|extends| self.lower_extends(extends));
        let implements = class.implements.as_ref().map(|implements| self.lower_implements(implements));
        let attribute_target = self.lower_attribute_target(name.value, &class.attribute_lists);

        let document = self.phpdoc_resolution.get(class.span());
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::ClassLike(name));
        let annotation = self.lower_item_annotation(document.as_ref(), Some(name));
        let members = self.lower_members(class.left_brace.join(class.right_brace), &class.members, name);
        self.type_resolution.leave_scope();

        self.arena.alloc(Class {
            span: class.span(),
            annotation,
            attributes,
            version_constraint,
            attribute_target,
            modifiers,
            name,
            extends,
            implements,
            members,
        })
    }
}
