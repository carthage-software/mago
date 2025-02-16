use bumpalo::boxed::Box;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;
use strum::Display;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;
use crate::sequence::TokenSeparatedSequence;

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct ArrayAccess<'a> {
    pub array: Box<'a, Expression<'a>>,
    pub left_bracket: Span,
    pub index: Box<'a, Expression<'a>>,
    pub right_bracket: Span,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct ArrayAppend<'a> {
    pub array: Box<'a, Expression<'a>>,
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct List<'a> {
    pub list: Keyword,
    pub left_parenthesis: Span,
    pub elements: TokenSeparatedSequence<'a, ArrayElement<'a>>,
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Array<'a> {
    pub left_bracket: Span,
    pub elements: TokenSeparatedSequence<'a, ArrayElement<'a>>,
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct LegacyArray<'a> {
    pub array: Keyword,
    pub left_parenthesis: Span,
    pub elements: TokenSeparatedSequence<'a, ArrayElement<'a>>,
    pub right_parenthesis: Span,
}

/// Represents an array element.
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum ArrayElement<'a> {
    KeyValue(KeyValueArrayElement<'a>),
    Value(ValueArrayElement<'a>),
    Variadic(VariadicArrayElement<'a>),
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct KeyValueArrayElement<'a> {
    pub key: Box<'a, Expression<'a>>,
    pub double_arrow: Span,
    pub value: Box<'a, Expression<'a>>,
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct ValueArrayElement<'a> {
    pub value: Box<'a, Expression<'a>>,
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct VariadicArrayElement<'a> {
    pub ellipsis: Span,
    pub value: Box<'a, Expression<'a>>,
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct MissingArrayElement {
    pub comma: Span,
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
