use serde::Deserialize;
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Method {
    pub attributes: Sequence<AttributeList>,
    pub modifiers: Sequence<Modifier>,
    pub function: Keyword,
    pub ampersand: Option<Span>,
    pub name: LocalIdentifier,
    pub parameters: FunctionLikeParameterList,
    pub return_type_hint: Option<FunctionLikeReturnTypeHint>,
    pub body: MethodBody,
}

/// Represents the body of a method statement in PHP.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
pub enum MethodBody {
    Abstract(MethodAbstractBody),
    Concrete(Block),
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct MethodAbstractBody {
    pub semicolon: Span,
}

impl Method {
    /// Returns `true` if the method contains any promoted properties.
    pub fn has_promoted_properties(&self) -> bool {
        self.parameters.parameters.iter().any(|parameter| parameter.is_promoted_property())
    }

    /// Returns `true` if the method is abstract.
    #[inline]
    pub const fn is_abstract(&self) -> bool {
        matches!(self.body, MethodBody::Abstract(_))
    }
}

impl HasSpan for Method {
    fn span(&self) -> Span {
        if let Some(attribute_list) = self.attributes.first() {
            return Span::between(attribute_list.span(), self.body.span());
        }

        if let Some(modifier) = self.modifiers.first() {
            return Span::between(modifier.span(), self.body.span());
        }

        Span::between(self.function.span, self.body.span())
    }
}

impl HasSpan for MethodBody {
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
