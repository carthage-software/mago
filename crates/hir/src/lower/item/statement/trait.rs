use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::item::annotation::generics::TypeParameterDefiningEntity;
use crate::ir::item::statement::r#trait::Trait;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_trait(&mut self, r#trait: &'scratch cst::Trait<'scratch>) -> &'arena Trait<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&r#trait.attribute_lists);
        let version_constraint = self.lower_version_constraint(&r#trait.attribute_lists);
        let name = self.lower_declaration_name(&r#trait.name);

        let document = self.phpdoc_resolution.get(r#trait.span());
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::ClassLike(name));
        let annotation = self.lower_item_annotation(document.as_ref(), Some(name));
        let members = self.lower_members(r#trait.left_brace.join(r#trait.right_brace), &r#trait.members, name);
        self.type_resolution.leave_scope();

        self.arena.alloc(Trait { span: r#trait.span(), annotation, attributes, version_constraint, name, members })
    }
}
