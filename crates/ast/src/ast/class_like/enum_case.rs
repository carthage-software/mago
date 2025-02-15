use serde::Deserialize;
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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct EnumCase {
    pub attribute_lists: Sequence<AttributeList>,
    pub case: Keyword,
    pub item: EnumCaseItem,
    pub terminator: Terminator,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum EnumCaseItem {
    Unit(EnumCaseUnitItem),
    Backed(EnumCaseBackedItem),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct EnumCaseUnitItem {
    pub name: LocalIdentifier,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct EnumCaseBackedItem {
    pub name: LocalIdentifier,
    pub equals: Span,
    pub value: Expression,
}

impl EnumCaseItem {
    pub fn name(&self) -> &LocalIdentifier {
        match &self {
            EnumCaseItem::Unit(enum_case_unit_item) => &enum_case_unit_item.name,
            EnumCaseItem::Backed(enum_case_backed_item) => &enum_case_backed_item.name,
        }
    }
}

impl HasSpan for EnumCase {
    fn span(&self) -> Span {
        if let Some(attribute_list) = self.attribute_lists.first() {
            return attribute_list.span().join(self.terminator.span());
        }

        self.case.span().join(self.terminator.span())
    }
}

impl HasSpan for EnumCaseItem {
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

impl HasSpan for EnumCaseBackedItem {
    fn span(&self) -> Span {
        Span::between(self.name.span(), self.value.span())
    }
}
