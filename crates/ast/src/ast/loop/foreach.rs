use bumpalo::boxed::Box;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;
use crate::ast::statement::Statement;
use crate::ast::terminator::Terminator;
use crate::sequence::Sequence;

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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Foreach<'a> {
    pub foreach: Keyword,
    pub left_parenthesis: Span,
    pub expression: Box<'a, Expression<'a>>,
    pub r#as: Keyword,
    pub target: ForeachTarget<'a>,
    pub right_parenthesis: Span,
    pub body: ForeachBody<'a>,
}

/// Represents the target of a foreach statement.
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum ForeachTarget<'a> {
    Value(ForeachValueTarget<'a>),
    KeyValue(ForeachKeyValueTarget<'a>),
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct ForeachValueTarget<'a> {
    pub value: Box<'a, Expression<'a>>,
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct ForeachKeyValueTarget<'a> {
    pub key: Box<'a, Expression<'a>>,
    pub double_arrow: Span,
    pub value: Box<'a, Expression<'a>>,
}

/// Represents the body of a foreach statement.
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum ForeachBody<'a> {
    /// The body is a statement.
    Statement(Box<'a, Statement<'a>>),
    /// The body is a colon-delimited body.
    ColonDelimited(ForeachColonDelimitedBody<'a>),
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct ForeachColonDelimitedBody<'a> {
    pub colon: Span,
    pub statements: Sequence<'a, Statement<'a>>,
    pub end_foreach: Keyword,
    pub terminator: Terminator,
}

impl HasSpan for Foreach<'_> {
    fn span(&self) -> Span {
        self.foreach.span().join(self.body.span())
    }
}

impl HasSpan for ForeachTarget<'_> {
    fn span(&self) -> Span {
        match self {
            ForeachTarget::Value(value) => value.span(),
            ForeachTarget::KeyValue(key_value) => key_value.span(),
        }
    }
}

impl HasSpan for ForeachValueTarget<'_> {
    fn span(&self) -> Span {
        self.value.span()
    }
}

impl HasSpan for ForeachKeyValueTarget<'_> {
    fn span(&self) -> Span {
        self.key.span().join(self.value.span())
    }
}

impl HasSpan for ForeachBody<'_> {
    fn span(&self) -> Span {
        match self {
            ForeachBody::Statement(statement) => statement.span(),
            ForeachBody::ColonDelimited(body) => body.span(),
        }
    }
}

impl HasSpan for ForeachColonDelimitedBody<'_> {
    fn span(&self) -> Span {
        self.colon.join(self.terminator.span())
    }
}
