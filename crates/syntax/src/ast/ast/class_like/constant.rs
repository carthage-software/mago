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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ClassLikeConstant<'arena> {
    pub attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    pub modifiers: Sequence<'arena, Modifier<'arena>>,
    pub r#const: Keyword<'arena>,
    pub hint: Option<Hint<'arena>>,
    pub items: TokenSeparatedSequence<'arena, ClassLikeConstantItem<'arena>>,
    pub terminator: Terminator<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ClassLikeConstantItem<'arena> {
    pub name: LocalIdentifier<'arena>,
    pub equals: Span,
    pub value: Expression<'arena>,
}

impl<'arena> ClassLikeConstant<'arena> {
    pub fn first_item(&self) -> &ClassLikeConstantItem<'arena> {
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
