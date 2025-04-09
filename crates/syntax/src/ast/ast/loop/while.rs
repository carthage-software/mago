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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct While {
    pub r#while: Keyword,
    pub left_parenthesis: Span,
    pub condition: Box<Expression>,
    pub right_parenthesis: Span,
    pub body: WhileBody,
}

/// Represents the body of a while statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum WhileBody {
    Statement(Box<Statement>),
    ColonDelimited(WhileColonDelimitedBody),
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct WhileColonDelimitedBody {
    pub colon: Span,
    pub statements: Sequence<Statement>,
    pub end_while: Keyword,
    pub terminator: Terminator,
}

impl HasSpan for While {
    fn span(&self) -> Span {
        self.r#while.span().join(self.body.span())
    }
}

impl HasSpan for WhileBody {
    fn span(&self) -> Span {
        match self {
            WhileBody::Statement(statement) => statement.span(),
            WhileBody::ColonDelimited(body) => body.span(),
        }
    }
}

impl HasSpan for WhileColonDelimitedBody {
    fn span(&self) -> Span {
        self.colon.join(self.terminator.span())
    }
}
