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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Foreach<'arena> {
    pub foreach: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub expression: &'arena Expression<'arena>,
    pub r#as: Keyword<'arena>,
    pub target: ForeachTarget<'arena>,
    pub right_parenthesis: Span,
    pub body: ForeachBody<'arena>,
}

/// Represents the target of a foreach statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum ForeachTarget<'arena> {
    Value(ForeachValueTarget<'arena>),
    KeyValue(ForeachKeyValueTarget<'arena>),
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ForeachValueTarget<'arena> {
    pub value: &'arena Expression<'arena>,
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ForeachKeyValueTarget<'arena> {
    pub key: &'arena Expression<'arena>,
    pub double_arrow: Span,
    pub value: &'arena Expression<'arena>,
}

/// Represents the body of a foreach statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum ForeachBody<'arena> {
    /// The body is a statement.
    Statement(&'arena Statement<'arena>),
    /// The body is a colon-delimited body.
    ColonDelimited(ForeachColonDelimitedBody<'arena>),
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ForeachColonDelimitedBody<'arena> {
    pub colon: Span,
    pub statements: Sequence<'arena, Statement<'arena>>,
    pub end_foreach: Keyword<'arena>,
    pub terminator: Terminator<'arena>,
}

impl<'arena> ForeachTarget<'arena> {
    pub fn key(&self) -> Option<&Expression<'arena>> {
        match self {
            ForeachTarget::Value(_) => None,
            ForeachTarget::KeyValue(key_value) => Some(key_value.key),
        }
    }

    pub fn value(&self) -> &Expression<'arena> {
        match self {
            ForeachTarget::Value(value) => value.value,
            ForeachTarget::KeyValue(key_value) => key_value.value,
        }
    }
}

impl<'arena> ForeachBody<'arena> {
    pub fn statements(&self) -> &[Statement<'arena>] {
        match self {
            ForeachBody::Statement(statement) => std::slice::from_ref(statement),
            ForeachBody::ColonDelimited(body) => body.statements.as_slice(),
        }
    }
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
