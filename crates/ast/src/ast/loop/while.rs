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

/// Represents a while statement in PHP.
///
/// Example:
///
/// ```php
/// <?php
///
/// $i = 0;
/// while ($i < 10) {
///   echo $i;
///   $i++;
/// }
/// ```
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct While<'a> {
    pub r#while: Keyword,
    pub left_parenthesis: Span,
    pub condition: Box<'a, Expression<'a>>,
    pub right_parenthesis: Span,
    pub body: WhileBody<'a>,
}

/// Represents the body of a while statement.
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum WhileBody<'a> {
    Statement(Box<'a, Statement<'a>>),
    ColonDelimited(WhileColonDelimitedBody<'a>),
}

/// Represents a colon-delimited body of a while statement.
///
/// Example:
///
/// ```php
/// <?php
///
/// $i = 0;
/// while ($i < 10):
///   echo $i;
///   $i++;
/// endwhile;
/// ```
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct WhileColonDelimitedBody<'a> {
    pub colon: Span,
    pub statements: Sequence<'a, Statement<'a>>,
    pub end_while: Keyword,
    pub terminator: Terminator,
}

impl HasSpan for While<'_> {
    fn span(&self) -> Span {
        self.r#while.span().join(self.body.span())
    }
}

impl HasSpan for WhileBody<'_> {
    fn span(&self) -> Span {
        match self {
            WhileBody::Statement(statement) => statement.span(),
            WhileBody::ColonDelimited(body) => body.span(),
        }
    }
}

impl HasSpan for WhileColonDelimitedBody<'_> {
    fn span(&self) -> Span {
        self.colon.join(self.terminator.span())
    }
}
