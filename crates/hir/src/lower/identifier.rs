use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::identifier::Identifier;
use crate::ir::identifier::IdentifierKind;
use crate::lower::Lowering;
use crate::lower::resolution::namespace::NameResolutionKind;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_identifier(
        &mut self,
        identifier: &'scratch cst::Identifier<'scratch>,
        resolve: Option<NameResolutionKind>,
    ) -> Identifier<'arena> {
        let kind = match identifier {
            cst::Identifier::Local(_) => IdentifierKind::Local,
            cst::Identifier::Qualified(_) => IdentifierKind::Qualified,
            cst::Identifier::FullyQualified(_) => IdentifierKind::FullyQualified,
        };

        let (value, imported) = match resolve {
            Some(name_kind) => self.namespace_resolution.resolve_name(name_kind, identifier.value()),
            None => (identifier.value(), false),
        };

        Identifier { span: identifier.span(), imported, value: self.interner.intern(value), kind }
    }

    pub(crate) fn lower_declaration_name(
        &mut self,
        name: &'scratch cst::LocalIdentifier<'scratch>,
    ) -> Identifier<'arena> {
        Identifier {
            span: name.span,
            imported: false,
            value: self.interner.intern(self.namespace_resolution.qualify(name.value)),
            kind: IdentifierKind::Local,
        }
    }
}
