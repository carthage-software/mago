use mago_span::HasSpan;
use mago_syntax::cst;
use mago_syntax::cst::Sequence;

use crate::ir::modifier::Modifier;
use crate::ir::modifier::ModifierKind;
use crate::lower::Lowering;

impl<'arena> Lowering<'_, 'arena> {
    pub(crate) fn lower_modifiers(&self, modifier: &Sequence<'arena, cst::Modifier<'arena>>) -> &'arena [Modifier] {
        self.arena.alloc_slice_fill_iter(modifier.iter().map(|modifier| self.lower_modifier(modifier)))
    }

    pub(crate) fn lower_modifier(&self, modifier: &'arena cst::Modifier<'arena>) -> Modifier {
        Modifier {
            span: modifier.span(),
            kind: match modifier {
                cst::Modifier::Static(_) => ModifierKind::Static,
                cst::Modifier::Final(_) => ModifierKind::Final,
                cst::Modifier::Abstract(_) => ModifierKind::Abstract,
                cst::Modifier::Readonly(_) => ModifierKind::Readonly,
                cst::Modifier::Public(_) => ModifierKind::Public,
                cst::Modifier::PublicSet(_) => ModifierKind::PublicSet,
                cst::Modifier::Protected(_) => ModifierKind::Protected,
                cst::Modifier::ProtectedSet(_) => ModifierKind::ProtectedSet,
                cst::Modifier::Private(_) => ModifierKind::Private,
                cst::Modifier::PrivateSet(_) => ModifierKind::PrivateSet,
            },
        }
    }
}
