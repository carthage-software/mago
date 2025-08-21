use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;
use crate::ast::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ArrayAccess<'arena> {
    pub array: &'arena Expression<'arena>,
    pub left_bracket: Span,
    pub index: &'arena Expression<'arena>,
    pub right_bracket: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ArrayAppend<'arena> {
    pub array: &'arena Expression<'arena>,
    pub left_bracket: Span,
    pub right_bracket: Span,
}

/// Represents a PHP list, defined using `list` keyword and parentheses `()`.
///
/// # Example:
///
/// ```php
/// <?php
///
/// list($a, 'b' => $c, /* missing */, ...$rest) = $arr;
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct List<'arena> {
    pub list: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub elements: TokenSeparatedSequence<'arena, ArrayElement<'arena>>,
    pub right_parenthesis: Span,
}

/// Represents a standard PHP array, defined using square brackets `[]`.
///
/// # Example:
///
/// ```php
/// <?php
///
/// $arr = ['apple', 'banana', 3 => 'orange'];
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Array<'arena> {
    pub left_bracket: Span,
    pub elements: TokenSeparatedSequence<'arena, ArrayElement<'arena>>,
    pub right_bracket: Span,
}

/// Represents a legacy PHP array, defined using `array` keyword and parentheses `()`.
///
/// # Example:
///
/// ```php
/// <?php
///
/// $arr = array('apple', 'banana', 3 => 'orange');
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct LegacyArray<'arena> {
    pub array: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub elements: TokenSeparatedSequence<'arena, ArrayElement<'arena>>,
    pub right_parenthesis: Span,
}

/// Represents an array element.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum ArrayElement<'arena> {
    KeyValue(KeyValueArrayElement<'arena>),
    Value(ValueArrayElement<'arena>),
    Variadic(VariadicArrayElement<'arena>),
    Missing(MissingArrayElement),
}

/// Represents a key-value pair in an array.
///
/// # Example:
///
/// ```php
/// <?php
///
/// $arr = [
///   1 => 'orange',
/// ];
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct KeyValueArrayElement<'arena> {
    pub key: &'arena Expression<'arena>,
    pub double_arrow: Span,
    pub value: &'arena Expression<'arena>,
}

/// Represents a value in an array.
///
/// # Example:
///
/// ```php
/// <?php
///
/// $arr = [
///   'orange',
/// ];
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ValueArrayElement<'arena> {
    pub value: &'arena Expression<'arena>,
}

/// Represents a variadic array element.
///
/// # Example:
///
/// ```php
/// <?php
///
/// $arr = [
///   ...$other,
/// ];
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct VariadicArrayElement<'arena> {
    pub ellipsis: Span,
    pub value: &'arena Expression<'arena>,
}

/// Represents a missing array element.
///
/// # Example:
///
/// ```php
/// <?php
///
/// $arr = [
///   'first',
///   ,
///   'third',
/// ];
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct MissingArrayElement {
    pub comma: Span,
}

impl<'arena> ArrayElement<'arena> {
    #[inline]
    pub const fn is_variadic(&self) -> bool {
        matches!(self, ArrayElement::Variadic(_))
    }

    #[inline]
    pub const fn is_missing(&self) -> bool {
        matches!(self, ArrayElement::Missing(_))
    }

    #[inline]
    pub const fn is_key_value(&self) -> bool {
        matches!(self, ArrayElement::KeyValue(_))
    }

    #[inline]
    pub const fn is_value(&self) -> bool {
        matches!(self, ArrayElement::Value(_))
    }

    #[inline]
    pub fn get_key(&self) -> Option<&Expression<'arena>> {
        match self {
            ArrayElement::KeyValue(element) => Some(element.key),
            ArrayElement::Value(_) => None,
            ArrayElement::Variadic(_) => None,
            ArrayElement::Missing(_) => None,
        }
    }

    #[inline]
    pub fn get_value(&self) -> Option<&Expression<'arena>> {
        match self {
            ArrayElement::KeyValue(element) => Some(element.value),
            ArrayElement::Value(element) => Some(element.value),
            ArrayElement::Variadic(element) => Some(element.value),
            ArrayElement::Missing(_) => None,
        }
    }
}

impl HasSpan for ArrayAccess<'_> {
    fn span(&self) -> Span {
        self.array.span().join(self.right_bracket)
    }
}

impl HasSpan for ArrayAppend<'_> {
    fn span(&self) -> Span {
        self.array.span().join(self.right_bracket)
    }
}

impl HasSpan for List<'_> {
    fn span(&self) -> Span {
        self.list.span().join(self.right_parenthesis)
    }
}

impl HasSpan for Array<'_> {
    fn span(&self) -> Span {
        self.left_bracket.join(self.right_bracket)
    }
}

impl HasSpan for LegacyArray<'_> {
    fn span(&self) -> Span {
        self.array.span().join(self.right_parenthesis)
    }
}

impl HasSpan for ArrayElement<'_> {
    fn span(&self) -> Span {
        match self {
            ArrayElement::KeyValue(element) => element.span(),
            ArrayElement::Value(element) => element.span(),
            ArrayElement::Variadic(element) => element.span(),
            ArrayElement::Missing(element) => element.span(),
        }
    }
}

impl HasSpan for KeyValueArrayElement<'_> {
    fn span(&self) -> Span {
        self.key.span().join(self.value.span())
    }
}

impl HasSpan for ValueArrayElement<'_> {
    fn span(&self) -> Span {
        self.value.span()
    }
}

impl HasSpan for VariadicArrayElement<'_> {
    fn span(&self) -> Span {
        self.ellipsis.join(self.value.span())
    }
}

impl HasSpan for MissingArrayElement {
    fn span(&self) -> Span {
        self.comma
    }
}
