use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::identifier::Identifier;
use crate::ir::item::inheritance::Extends;
use crate::ir::item::inheritance::Implements;
use crate::lower::Lowering;
use crate::lower::resolution::namespace::NameResolutionKind;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_extends(&mut self, extends: &'scratch cst::Extends<'scratch>) -> &'arena Extends<'arena> {
        let span = extends.span();
        let types = self.lower_class_reference_list(&extends.types);

        self.arena.alloc(Extends { span, types })
    }

    pub(crate) fn lower_implements(
        &mut self,
        implements: &'scratch cst::Implements<'scratch>,
    ) -> &'arena Implements<'arena> {
        let span = implements.span();
        let types = self.lower_class_reference_list(&implements.types);

        self.arena.alloc(Implements { span, types })
    }

    pub(crate) fn lower_class_reference_list(
        &mut self,
        types: &'scratch cst::TokenSeparatedSequence<'scratch, cst::Identifier<'scratch>>,
    ) -> &'arena [Identifier<'arena>] {
        self.arena.alloc_slice_fill_iter(
            types.iter().map(|identifier| self.lower_identifier(identifier, Some(NameResolutionKind::Default))),
        )
    }
}
