use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::attribute::AttributeList;
use crate::ast::ast::block::Block;
use crate::ast::ast::function_like::parameter::FunctionLikeParameterList;
use crate::ast::ast::function_like::r#return::FunctionLikeReturnTypeHint;
use crate::ast::ast::identifier::LocalIdentifier;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::modifier::Modifier;
use crate::ast::sequence::Sequence;

/// Represents a method statement in PHP.
///
/// Example:
///
/// ```php
/// class Foo {
///    public function bar() {
///       return 'baz';
///    }
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Method<'arena> {
    pub attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    pub modifiers: Sequence<'arena, Modifier<'arena>>,
    pub function: Keyword<'arena>,
    pub ampersand: Option<Span>,
    pub name: LocalIdentifier<'arena>,
    pub parameter_list: FunctionLikeParameterList<'arena>,
    pub return_type_hint: Option<FunctionLikeReturnTypeHint<'arena>>,
    pub body: MethodBody<'arena>,
}

/// Represents the body of a method statement in PHP.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum MethodBody<'arena> {
    Abstract(MethodAbstractBody),
    Concrete(Block<'arena>),
}

/// Represents the abstract body of a method statement in PHP.
///
/// Example:
///
/// ```php
/// <?php
///
/// abstract class Foo {
///   abstract public function bar();
/// }
///
/// ```
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct MethodAbstractBody {
    pub semicolon: Span,
}

impl Method<'_> {
    /// Returns `true` if the method contains any promoted properties.
    pub fn has_promoted_properties(&self) -> bool {
        self.parameter_list.parameters.iter().any(|parameter| parameter.is_promoted_property())
    }

    /// Returns `true` if the method is abstract.
    #[inline]
    pub const fn is_abstract(&self) -> bool {
        matches!(self.body, MethodBody::Abstract(_))
    }

    /// Returns `true` if the method is static.
    #[inline]
    pub fn is_static(&self) -> bool {
        self.modifiers.iter().any(|modifier| modifier.is_static())
    }
}

impl HasSpan for Method<'_> {
    fn span(&self) -> Span {
        if let Some(attribute_list) = self.attribute_lists.first() {
            return Span::between(attribute_list.span(), self.body.span());
        }

        if let Some(modifier) = self.modifiers.first() {
            return Span::between(modifier.span(), self.body.span());
        }

        Span::between(self.function.span, self.body.span())
    }
}

impl HasSpan for MethodBody<'_> {
    fn span(&self) -> Span {
        match self {
            MethodBody::Abstract(body) => body.span(),
            MethodBody::Concrete(body) => body.span(),
        }
    }
}

impl HasSpan for MethodAbstractBody {
    fn span(&self) -> Span {
        self.semicolon
    }
}
