use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::statement::Statement;
use crate::ast::ast::terminator::Terminator;
use crate::ast::sequence::Sequence;

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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct While<'arena> {
    pub r#while: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub condition: &'arena Expression<'arena>,
    pub right_parenthesis: Span,
    pub body: WhileBody<'arena>,
}

/// Represents the body of a while statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum WhileBody<'arena> {
    Statement(&'arena Statement<'arena>),
    ColonDelimited(WhileColonDelimitedBody<'arena>),
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct WhileColonDelimitedBody<'arena> {
    pub colon: Span,
    pub statements: Sequence<'arena, Statement<'arena>>,
    pub end_while: Keyword<'arena>,
    pub terminator: Terminator<'arena>,
}

impl<'arena> WhileBody<'arena> {
    #[inline]
    pub fn statements(&self) -> &[Statement<'arena>] {
        match self {
            WhileBody::Statement(statement) => std::slice::from_ref(statement),
            WhileBody::ColonDelimited(body) => body.statements.as_slice(),
        }
    }
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
