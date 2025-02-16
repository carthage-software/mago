use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::attribute::AttributeList;
use crate::ast::block::Block;
use crate::ast::function_like::parameter::FunctionLikeParameterList;
use crate::ast::function_like::r#return::FunctionLikeReturnTypeHint;
use crate::ast::identifier::LocalIdentifier;
use crate::ast::keyword::Keyword;
use crate::ast::modifier::Modifier;
use crate::sequence::Sequence;

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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Method<'a> {
    pub attribute_lists: Sequence<'a, AttributeList<'a>>,
    pub modifiers: Sequence<'a, Modifier>,
    pub function: Keyword,
    pub ampersand: Option<Span>,
    pub name: LocalIdentifier,
    pub parameter_list: FunctionLikeParameterList<'a>,
    pub return_type_hint: Option<FunctionLikeReturnTypeHint<'a>>,
    pub body: MethodBody<'a>,
}

/// Represents the body of a method statement in PHP.
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum MethodBody<'a> {
    Abstract(MethodAbstractBody),
    Concrete(Block<'a>),
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
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
