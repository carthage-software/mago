use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;

/// Represents a PHP `yield` expression.
///
/// # Examples
///
/// ```php
/// <?php
///
/// function gen(): Generator {
///     yield 1;
///     yield 2 => 3;
///     yield from [4, 5];
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum Yield {
    Value(YieldValue),
    Pair(YieldPair),
    From(YieldFrom),
}

/// Represents a PHP `yield` expression with a value.
///
/// # Examples
///
/// ```php
/// <?php
///
/// function gen(): Generator {
///    yield 1;
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct YieldValue {
    pub r#yield: Keyword,
    pub value: Option<Box<Expression>>,
}

/// Represents a PHP `yield` expression with a key-value pair.
///
/// # Examples
///
/// ```php
/// <?php
///
/// function gen(): Generator {
///   yield 2 => 3;
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct YieldPair {
    pub r#yield: Keyword,
    pub key: Box<Expression>,
    pub arrow: Span,
    pub value: Box<Expression>,
}

/// Represents a PHP `yield from` expression.
///
/// # Examples
///
/// ```php
/// <?php
///
/// function gen(): Generator {
///  yield from [4, 5];
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct YieldFrom {
    pub r#yield: Keyword,
    pub from: Keyword,
    pub iterator: Box<Expression>,
}

impl HasSpan for Yield {
    fn span(&self) -> Span {
        match self {
            Yield::Value(y) => y.span(),
            Yield::Pair(y) => y.span(),
            Yield::From(y) => y.span(),
        }
    }
}

impl HasSpan for YieldValue {
    fn span(&self) -> Span {
        if let Some(value) = &self.value {
            self.r#yield.span().join(value.span())
        } else {
            self.r#yield.span()
        }
    }
}

impl HasSpan for YieldPair {
    fn span(&self) -> Span {
        self.r#yield.span().join(self.value.span())
    }
}

impl HasSpan for YieldFrom {
    fn span(&self) -> Span {
        self.r#yield.span().join(self.iterator.span())
    }
}
