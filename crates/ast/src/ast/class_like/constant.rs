use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::attribute::AttributeList;
use crate::ast::expression::Expression;
use crate::ast::identifier::LocalIdentifier;
use crate::ast::keyword::Keyword;
use crate::ast::modifier::Modifier;
use crate::ast::terminator::Terminator;
use crate::ast::type_hint::Hint;
use crate::sequence::Sequence;
use crate::sequence::TokenSeparatedSequence;

/// Represents a class-like constant in PHP.
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct ClassLikeConstant<'a> {
    pub attribute_lists: Sequence<'a, AttributeList<'a>>,
    pub modifiers: Sequence<'a, Modifier>,
    pub r#const: Keyword,
    pub hint: Option<Hint<'a>>,
    pub items: TokenSeparatedSequence<'a, ClassLikeConstantItem<'a>>,
    pub terminator: Terminator,
}

/// Represents a single name-value pair within a constant statement.
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct ClassLikeConstantItem<'a> {
    pub name: LocalIdentifier,
    pub equals: Span,
    pub value: Expression<'a>,
}

impl<'a> ClassLikeConstant<'a> {
    pub fn first_item(&self) -> &ClassLikeConstantItem<'a> {
        self.items
            .first()
            .expect("expected class-like constant to have at least 1 item. this is a bug in mago. please report it.")
    }
}

impl HasSpan for ClassLikeConstant<'_> {
    fn span(&self) -> Span {
        if let Some(modifier) = self.modifiers.first() {
            modifier.span().join(self.terminator.span())
        } else {
            self.r#const.span().join(self.terminator.span())
        }
    }
}

impl HasSpan for ClassLikeConstantItem<'_> {
    fn span(&self) -> Span {
        self.name.span().join(self.value.span())
    }
}
