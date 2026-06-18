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
        // Constant and plain-property declarations can declare several members at once
        // (`const A = 1, B = 2;` / `public $a, $b;`); each declarator becomes its own
        // member node, in source order, so the result expands one CST member into many.
        let mut collected: Vec<MemberItem<'arena, (), (), ()>> = Vec::with_capacity(members.len());
        for member in members.iter() {
            match member {
                cst::ClassLikeMember::Method(method) => {
                    let node = self.arena.alloc(self.lower_method(method, owner));
                    collected.push(MemberItem { meta: (), span: member.span(), kind: MemberItemKind::Method(node) });
                }
                cst::ClassLikeMember::Property(cst::Property::Plain(property)) => {
                    for lowered in self.lower_plain_property(property) {
                        let span = lowered.span;
                        let node = self.arena.alloc(lowered);
                        collected.push(MemberItem { meta: (), span, kind: MemberItemKind::Property(node) });
                    }
                }
                cst::ClassLikeMember::Property(cst::Property::Hooked(property)) => {
                    let node = self.arena.alloc(self.lower_hooked_property(property));
                    collected.push(MemberItem {
                        meta: (),
                        span: member.span(),
                        kind: MemberItemKind::HookedProperty(node),
                    });
                }
                cst::ClassLikeMember::Constant(constant) => {
                    for lowered in self.lower_class_like_constant(constant) {
                        let span = lowered.span;
                        let node = self.arena.alloc(lowered);
                        collected.push(MemberItem { meta: (), span, kind: MemberItemKind::Constant(node) });
                    }
                }
                cst::ClassLikeMember::EnumCase(enum_case) => {
                    let node = self.arena.alloc(self.lower_enum_case(enum_case));
                    collected.push(MemberItem { meta: (), span: member.span(), kind: MemberItemKind::EnumCase(node) });
                }
                cst::ClassLikeMember::TraitUse(trait_use) => {
                    let node = self.arena.alloc(self.lower_trait_use(trait_use));
                    collected.push(MemberItem { meta: (), span: member.span(), kind: MemberItemKind::TraitUse(node) });
                }
            }
        }

        Delimited { span, items: self.arena.alloc_slice_copy(&collected) }
    }
}
