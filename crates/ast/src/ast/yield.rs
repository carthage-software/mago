use bumpalo::boxed::Box;
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
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum Yield<'a> {
    Value(YieldValue<'a>),
    Pair(YieldPair<'a>),
    From(YieldFrom<'a>),
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct YieldValue<'a> {
    pub r#yield: Keyword,
    pub value: Option<Box<'a, Expression<'a>>>,
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct YieldPair<'a> {
    pub r#yield: Keyword,
    pub key: Box<'a, Expression<'a>>,
    pub arrow: Span,
    pub value: Box<'a, Expression<'a>>,
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct YieldFrom<'a> {
    pub r#yield: Keyword,
    pub from: Keyword,
    pub iterator: Box<'a, Expression<'a>>,
}

impl HasSpan for Yield<'_> {
    fn span(&self) -> Span {
        match self {
            Yield::Value(y) => y.span(),
            Yield::Pair(y) => y.span(),
            Yield::From(y) => y.span(),
        }
    }
}

impl HasSpan for YieldValue<'_> {
    fn span(&self) -> Span {
        if let Some(value) = &self.value {
            self.r#yield.span().join(value.span())
        } else {
            self.r#yield.span()
        }
    }
}

impl HasSpan for YieldPair<'_> {
    fn span(&self) -> Span {
        self.r#yield.span().join(self.value.span())
    }
}

impl HasSpan for YieldFrom<'_> {
    fn span(&self) -> Span {
        self.r#yield.span().join(self.iterator.span())
    }
}
