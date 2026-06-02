use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::identifier::Identifier;
use crate::cst::keyword::Keyword;
use crate::cst::r#type::Type;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TypeAliasTagValue<'arena> {
    pub alias: Identifier<'arena>,
    pub equals: Option<Span>,
    pub r#type: &'arena Type<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TypeAliasImportTagValue<'arena> {
    pub imported_alias: Identifier<'arena>,
    pub from_keyword: Keyword<'arena>,
    pub imported_from: Identifier<'arena>,
    pub imported_as: Option<TypeAliasImportTagValueAs<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TypeAliasImportTagValueAs<'arena> {
    pub keyword: Keyword<'arena>,
    pub local: Identifier<'arena>,
}

impl HasSpan for TypeAliasTagValue<'_> {
    fn span(&self) -> Span {
        self.alias.span().join(self.r#type.span())
    }
}

impl HasSpan for TypeAliasImportTagValue<'_> {
    fn span(&self) -> Span {
        let end = self.imported_as.as_ref().map_or_else(|| self.imported_from.span(), HasSpan::span);

        self.imported_alias.span().join(end)
    }
}

impl HasSpan for TypeAliasImportTagValueAs<'_> {
    fn span(&self) -> Span {
        self.keyword.span().join(self.local.span())
    }
}
