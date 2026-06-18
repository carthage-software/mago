use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::item::annotation::generics::TypeParameterDefiningEntity;
use crate::ir::item::statement::r#enum::Enum;
use crate::ir::item::statement::r#enum::EnumBackingType;
use crate::ir::item::statement::r#enum::EnumBackingTypeKind;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    fn lower_enum_backing_type_kind(&mut self, hint: &'scratch cst::Hint<'scratch>) -> EnumBackingTypeKind<'arena> {
        match hint {
            cst::Hint::String(_) => EnumBackingTypeKind::String,
            cst::Hint::Integer(_) => EnumBackingTypeKind::Int,
            other => EnumBackingTypeKind::Invalid(self.lower_type(other)),
        }
    }

    pub(crate) fn lower_enum(&mut self, r#enum: &'scratch cst::Enum<'scratch>) -> &'arena Enum<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&r#enum.attribute_lists);
        let version_constraint = self.lower_version_constraint(&r#enum.attribute_lists);
        let name = self.lower_declaration_name(&r#enum.name);
        let backing_type = r#enum
            .backing_type_hint
            .as_ref()
            .map(|hint| EnumBackingType { span: hint.span(), kind: self.lower_enum_backing_type_kind(&hint.hint) });

        let implements = r#enum.implements.as_ref().map(|implements| self.lower_implements(implements));

        let document = self.phpdoc_resolution.get(r#enum.span());
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::ClassLike(name));
        let annotation = self.lower_item_annotation(document.as_ref(), Some(name));
        let members = self.lower_members(r#enum.left_brace.join(r#enum.right_brace), &r#enum.members, name);
        self.type_resolution.leave_scope();

        self.arena.alloc(Enum {
            span: r#enum.span(),
            annotation,
            attributes,
            version_constraint,
            name,
            backing_type,
            implements,
            members,
        })
    }
}
