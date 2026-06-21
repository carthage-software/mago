use mago_allocator::Arena;

use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::identifier::Identifier;
use crate::ir::identifier::IdentifierKind;
use crate::ir::item::annotation::generics::TypeParameterDefiningEntity;
use crate::ir::item::expression::anonymous_class::AnonymousClass;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_anonymous_class(
        &mut self,
        anonymous_class: &'scratch cst::AnonymousClass<'scratch>,
    ) -> &'arena AnonymousClass<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&anonymous_class.attribute_lists);
        let arguments =
            anonymous_class.argument_list.as_ref().map(|argument_list| self.lower_partial_argument_list(argument_list));

        let extends = anonymous_class.extends.as_ref().map(|extends| self.lower_extends(extends));
        let implements = anonymous_class.implements.as_ref().map(|implements| self.lower_implements(implements));

        let span = anonymous_class.span();
        let modifiers = self.lower_modifiers(&anonymous_class.modifiers);
        let name = self.build_synthetic_name(b"anonymous-class", span);
        let owner = Identifier { span, imported: false, value: name, kind: IdentifierKind::Local };
        let document = self.phpdoc_resolution.get(anonymous_class.span());
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::ClassLike(owner));
        let annotation = self.lower_item_annotation(document.as_ref(), Some(owner));
        let members = self.lower_members(
            anonymous_class.left_brace.join(anonymous_class.right_brace),
            &anonymous_class.members,
            owner,
        );

        self.type_resolution.leave_scope();

        self.arena.alloc(AnonymousClass {
            span,
            modifiers,
            name,
            annotation,
            attributes,
            arguments,
            extends,
            implements,
            members,
        })
    }
}
