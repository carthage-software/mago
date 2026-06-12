pub mod constant;
pub mod enum_case;
pub mod hook;
pub mod method;
pub mod property;
pub mod trait_use;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst;

use crate::ir::delimited::Delimited;
use crate::ir::identifier::Identifier;
use crate::ir::item::member::MemberItem;
use crate::ir::item::member::MemberItemKind;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_members(
        &mut self,
        span: Span,
        members: &'scratch cst::Sequence<'scratch, cst::ClassLikeMember<'scratch>>,
        owner: Identifier<'arena>,
    ) -> Delimited<'arena, MemberItem<'arena, (), (), ()>> {
        let items = self.arena.alloc_slice_fill_iter(members.iter().map(|member| {
            let span = member.span();
            let kind = match member {
                cst::ClassLikeMember::Method(method) => {
                    MemberItemKind::Method(self.arena.alloc(self.lower_method(method, owner)))
                }
                cst::ClassLikeMember::Property(cst::Property::Plain(property)) => {
                    MemberItemKind::Property(self.arena.alloc(self.lower_plain_property(property)))
                }
                cst::ClassLikeMember::Property(cst::Property::Hooked(property)) => {
                    MemberItemKind::HookedProperty(self.arena.alloc(self.lower_hooked_property(property)))
                }
                cst::ClassLikeMember::Constant(constant) => {
                    MemberItemKind::Constant(self.arena.alloc(self.lower_class_like_constant(constant)))
                }
                cst::ClassLikeMember::EnumCase(enum_case) => {
                    MemberItemKind::EnumCase(self.arena.alloc(self.lower_enum_case(enum_case)))
                }
                cst::ClassLikeMember::TraitUse(trait_use) => {
                    MemberItemKind::TraitUse(self.arena.alloc(self.lower_trait_use(trait_use)))
                }
            };

            MemberItem { meta: (), span, kind }
        }));

        Delimited { span, items }
    }
}
