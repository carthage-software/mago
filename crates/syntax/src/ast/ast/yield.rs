use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;

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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum Yield<'arena> {
    Value(YieldValue<'arena>),
    Pair(YieldPair<'arena>),
    From(YieldFrom<'arena>),
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct YieldValue<'arena> {
    pub r#yield: Keyword<'arena>,
    pub value: Option<&'arena Expression<'arena>>,
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct YieldPair<'arena> {
    pub r#yield: Keyword<'arena>,
    pub key: &'arena Expression<'arena>,
    pub arrow: Span,
    pub value: &'arena Expression<'arena>,
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct YieldFrom<'arena> {
    pub r#yield: Keyword<'arena>,
    pub from: Keyword<'arena>,
    pub iterator: &'arena Expression<'arena>,
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
        if let Some(value) = &self.value { self.r#yield.span().join(value.span()) } else { self.r#yield.span() }
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
