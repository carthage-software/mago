use bumpalo::boxed::Box;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::attribute::AttributeList;
use crate::ast::expression::Expression;
use crate::ast::identifier::LocalIdentifier;
use crate::ast::keyword::Keyword;
use crate::ast::terminator::Terminator;
use crate::sequence::Sequence;

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct EnumCase<'a> {
    pub attribute_lists: Sequence<'a, AttributeList<'a>>,
    pub case: Keyword,
    pub item: EnumCaseItem<'a>,
    pub terminator: Terminator,
}

#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum EnumCaseItem<'a> {
    Unit(EnumCaseUnitItem),
    Backed(EnumCaseBackedItem<'a>),
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct EnumCaseUnitItem {
    pub name: LocalIdentifier,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct EnumCaseBackedItem<'a> {
    pub name: LocalIdentifier,
    pub equals: Span,
    pub value: Box<'a, Expression<'a>>,
}

impl EnumCaseItem<'_> {
    pub fn name(&self) -> &LocalIdentifier {
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

impl HasSpan for EnumCaseUnitItem {
    fn span(&self) -> Span {
        self.name.span()
    }
}

impl HasSpan for EnumCaseBackedItem<'_> {
    fn span(&self) -> Span {
        Span::between(self.name.span(), self.value.span())
    }
}
