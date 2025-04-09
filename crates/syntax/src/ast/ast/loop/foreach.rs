use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::statement::Statement;
use crate::ast::ast::terminator::Terminator;
use crate::ast::sequence::Sequence;

/// Represents a foreach statement in PHP.
///
/// Example:
///
/// ```php
/// <?php
///
/// foreach ($array as $value) {
///    echo $value;
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Foreach {
    pub foreach: Keyword,
    pub left_parenthesis: Span,
    pub expression: Box<Expression>,
    pub r#as: Keyword,
    pub target: ForeachTarget,
    pub right_parenthesis: Span,
    pub body: ForeachBody,
}

/// Represents the target of a foreach statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum ForeachTarget {
    Value(ForeachValueTarget),
    KeyValue(ForeachKeyValueTarget),
}

/// Represents the target of a foreach statement that only assigns the value.
///
/// Example:
///
/// ```php
/// <?php
///
/// foreach ($array as $value) {
///   echo $value;
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ForeachValueTarget {
    pub value: Box<Expression>,
}

/// Represents the target of a foreach statement that assigns both the key and value.
///
/// Example:
///
/// ```php
/// <?php
///
/// foreach ($array as $key => $value) {
///   echo $key . ' => ' . $value . PHP_EOL;
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ForeachKeyValueTarget {
    pub key: Box<Expression>,
    pub double_arrow: Span,
    pub value: Box<Expression>,
}

/// Represents the body of a foreach statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum ForeachBody {
    /// The body is a statement.
    Statement(Box<Statement>),
    /// The body is a colon-delimited body.
    ColonDelimited(ForeachColonDelimitedBody),
}

/// Represents a colon-delimited body of a foreach statement.
///
/// Example:
///
/// ```php
/// <?php
///
/// foreach ($array as $value):
///   echo $value;
/// endforeach;
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ForeachColonDelimitedBody {
    pub colon: Span,
    pub statements: Sequence<Statement>,
    pub end_foreach: Keyword,
    pub terminator: Terminator,
}

impl HasSpan for Foreach {
    fn span(&self) -> Span {
        self.foreach.span().join(self.body.span())
    }
}

impl HasSpan for ForeachTarget {
    fn span(&self) -> Span {
        match self {
            ForeachTarget::Value(value) => value.span(),
            ForeachTarget::KeyValue(key_value) => key_value.span(),
        }
    }
}

impl HasSpan for ForeachValueTarget {
    fn span(&self) -> Span {
        self.value.span()
    }
}

impl HasSpan for ForeachKeyValueTarget {
    fn span(&self) -> Span {
        self.key.span().join(self.value.span())
    }
}

impl HasSpan for ForeachBody {
    fn span(&self) -> Span {
        match self {
            ForeachBody::Statement(statement) => statement.span(),
            ForeachBody::ColonDelimited(body) => body.span(),
        }
    }
}

impl HasSpan for ForeachColonDelimitedBody {
    fn span(&self) -> Span {
        self.colon.join(self.terminator.span())
    }
}
