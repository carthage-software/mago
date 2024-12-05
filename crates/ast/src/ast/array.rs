use serde::Deserialize;
use serde::Serialize;

use fennec_span::HasSpan;
use fennec_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;
use crate::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ArrayAccess {
    pub array: Box<Expression>,
    pub left_bracket: Span,
    pub index: Box<Expression>,
    pub right_bracket: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ArrayAppend {
    pub array: Box<Expression>,
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct List {
    pub list: Keyword,
    pub left_parenthesis: Span,
    pub elements: TokenSeparatedSequence<ArrayElement>,
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Array {
    pub left_bracket: Span,
    pub elements: TokenSeparatedSequence<ArrayElement>,
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct LegacyArray {
    pub array: Keyword,
    pub left_parenthesis: Span,
    pub elements: TokenSeparatedSequence<ArrayElement>,
    pub right_parenthesis: Span,
}

/// Represents an array element.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum ArrayElement {
    KeyValue(KeyValueArrayElement),
    Value(ValueArrayElement),
    Variadic(VariadicArrayElement),
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct KeyValueArrayElement {
    pub key: Box<Expression>,
    pub double_arrow: Span,
    pub value: Box<Expression>,
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ValueArrayElement {
    pub value: Box<Expression>,
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct VariadicArrayElement {
    pub ellipsis: Span,
    pub value: Box<Expression>,
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct MissingArrayElement {
    pub comma: Span,
}

impl HasSpan for ArrayAccess {
    fn span(&self) -> Span {
        self.array.span().join(self.right_bracket)
    }
}

impl HasSpan for ArrayAppend {
    fn span(&self) -> Span {
        self.array.span().join(self.right_bracket)
    }
}

impl HasSpan for List {
    fn span(&self) -> Span {
        self.list.span().join(self.right_parenthesis)
    }
}

impl HasSpan for Array {
    fn span(&self) -> Span {
        self.left_bracket.join(self.right_bracket)
    }
}

impl HasSpan for LegacyArray {
    fn span(&self) -> Span {
        self.array.span().join(self.right_parenthesis)
    }
}

impl HasSpan for ArrayElement {
    fn span(&self) -> Span {
        match self {
            ArrayElement::KeyValue(element) => element.span(),
            ArrayElement::Value(element) => element.span(),
            ArrayElement::Variadic(element) => element.span(),
            ArrayElement::Missing(element) => element.span(),
        }
    }
}

impl HasSpan for KeyValueArrayElement {
    fn span(&self) -> Span {
        self.key.span().join(self.value.span())
    }
}

impl HasSpan for ValueArrayElement {
    fn span(&self) -> Span {
        self.value.span()
    }
}

impl HasSpan for VariadicArrayElement {
    fn span(&self) -> Span {
        self.ellipsis.join(self.value.span())
    }
}

impl HasSpan for MissingArrayElement {
    fn span(&self) -> Span {
        self.comma
    }
}
