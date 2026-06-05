pub mod annotation;

use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::identifier::Identifier;
use crate::ir::inheritance::ExtendsOne;
use crate::ir::inheritance::ExtendsOneOrMore;
use crate::ir::inheritance::Implements;
use crate::lower::Lowering;
use crate::lower::resolution::namespace::NameResolutionKind;

impl<'arena> Lowering<'_, 'arena> {
    pub(crate) fn lower_extends_one(
        &self,
        extends: &'arena cst::Extends<'arena>,
    ) -> Option<&'arena ExtendsOne<'arena>> {
        let parent = extends.types.first()?;
        let r#type = self.lower_identifier(parent, Some(NameResolutionKind::Default));

        Some(self.arena.alloc(ExtendsOne { span: extends.span(), r#type }))
    }

    pub(crate) fn lower_extends_one_or_more(
        &self,
        extends: &'arena cst::Extends<'arena>,
    ) -> &'arena ExtendsOneOrMore<'arena> {
        let span = extends.span();
        let types = self.lower_class_reference_list(&extends.types);

        self.arena.alloc(ExtendsOneOrMore { span, types })
    }

    pub(crate) fn lower_implements(&self, implements: &'arena cst::Implements<'arena>) -> &'arena Implements<'arena> {
        let span = implements.span();
        let types = self.lower_class_reference_list(&implements.types);

        self.arena.alloc(Implements { span, types })
    }

    pub(crate) fn lower_class_reference_list(
        &self,
        types: &'arena cst::TokenSeparatedSequence<'arena, cst::Identifier<'arena>>,
    ) -> &'arena [Identifier<'arena>] {
        self.arena.alloc_slice_fill_iter(
            types.iter().map(|identifier| self.lower_identifier(identifier, Some(NameResolutionKind::Default))),
        )
    }
}
