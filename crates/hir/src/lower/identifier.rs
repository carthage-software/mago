use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::identifier::Identifier;
use crate::ir::identifier::IdentifierKind;
use crate::lower::Lowering;
use crate::lower::resolution::namespace::NameResolutionKind;

impl<'arena> Lowering<'_, 'arena> {
    pub(crate) fn lower_identifier(
        &self,
        identifier: &'arena cst::Identifier<'arena>,
        resolve: Option<NameResolutionKind>,
    ) -> Identifier<'arena> {
        let kind = match identifier {
            cst::Identifier::Local(_) => IdentifierKind::Local,
            cst::Identifier::Qualified(_) => IdentifierKind::Qualified,
            cst::Identifier::FullyQualified(_) => IdentifierKind::FullyQualified,
        };

        let value = match resolve {
            Some(name_kind) => self.namespace_resolution.resolve_name(name_kind, identifier.value()),
            None => identifier.value(),
        };

        Identifier { span: identifier.span(), value, kind }
    }

    pub(crate) fn lower_declaration_name(&self, name: &'arena cst::LocalIdentifier<'arena>) -> Identifier<'arena> {
        Identifier {
            span: name.span,
            value: self.namespace_resolution.qualify(name.value),
            kind: IdentifierKind::Local,
        }
    }
}
