use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::attribute::AttributeList;
use crate::ast::ast::expression::Expression;
use crate::ast::ast::identifier::LocalIdentifier;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::terminator::Terminator;
use crate::ast::sequence::Sequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct EnumCase<'arena> {
    pub attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    pub case: Keyword<'arena>,
    pub item: EnumCaseItem<'arena>,
    pub terminator: Terminator<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum EnumCaseItem<'arena> {
    Unit(EnumCaseUnitItem<'arena>),
    Backed(EnumCaseBackedItem<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct EnumCaseUnitItem<'arena> {
    pub name: LocalIdentifier<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct EnumCaseBackedItem<'arena> {
    pub name: LocalIdentifier<'arena>,
    pub equals: Span,
    pub value: Expression<'arena>,
}

impl<'arena> EnumCaseItem<'arena> {
    pub fn name(&self) -> &LocalIdentifier<'arena> {
        match &self {
            EnumCaseItem::Unit(enum_case_unit_item) => &enum_case_unit_item.name,
            EnumCaseItem::Backed(enum_case_backed_item) => &enum_case_backed_item.name,
        }
    }
}

impl HasSpan for EnumCase<'_> {
    fn span(&self) -> Span {
        if let Some(attribute_list) = self.attribute_lists.first() {
            return attribute_list.span().join(self.terminator.span());
        }

        self.case.span().join(self.terminator.span())
    }
}

impl HasSpan for EnumCaseItem<'_> {
    fn span(&self) -> Span {
        match self {
            EnumCaseItem::Unit(item) => item.span(),
            EnumCaseItem::Backed(item) => item.span(),
        }
    }
}

impl HasSpan for EnumCaseUnitItem<'_> {
    fn span(&self) -> Span {
        self.name.span()
    }
}

impl HasSpan for EnumCaseBackedItem<'_> {
    fn span(&self) -> Span {
        Span::between(self.name.span(), self.value.span())
    }
}
