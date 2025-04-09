use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::attribute::AttributeList;
use crate::ast::ast::expression::Expression;
use crate::ast::ast::identifier::LocalIdentifier;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::modifier::Modifier;
use crate::ast::ast::terminator::Terminator;
use crate::ast::ast::type_hint::Hint;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;

/// Represents a class-like constant in PHP.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ClassLikeConstant {
    pub attribute_lists: Sequence<AttributeList>,
    pub modifiers: Sequence<Modifier>,
    pub r#const: Keyword,
    pub hint: Option<Hint>,
    pub items: TokenSeparatedSequence<ClassLikeConstantItem>,
    pub terminator: Terminator,
}

/// Represents a single name-value pair within a constant statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ClassLikeConstantItem {
    pub name: LocalIdentifier,
    pub equals: Span,
    pub value: Expression,
}

impl ClassLikeConstant {
    pub fn first_item(&self) -> &ClassLikeConstantItem {
        self.items
            .first()
            .expect("expected class-like constant to have at least 1 item. this is a bug in mago. please report it.")
    }
}

impl HasSpan for ClassLikeConstant {
    fn span(&self) -> Span {
        if let Some(modifier) = self.modifiers.first() {
            modifier.span().join(self.terminator.span())
        } else {
            self.r#const.span().join(self.terminator.span())
        }
    }
}

impl HasSpan for ClassLikeConstantItem {
    fn span(&self) -> Span {
        self.name.span().join(self.value.span())
    }
}
