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

/// Represents an `if` statement.
///
/// # Examples
///
/// ```php
/// if ($a) {
///   echo "a is true";
/// } elseif ($b) {
///   echo "b is true";
/// } else {
///   echo "a and b are false";
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct If {
    pub r#if: Keyword,
    pub left_parenthesis: Span,
    pub condition: Box<Expression>,
    pub right_parenthesis: Span,
    pub body: IfBody,
}

/// Represents the body of an `if` statement.
///
/// This can be either a statement body or a colon-delimited body.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum IfBody {
    Statement(IfStatementBody),
    ColonDelimited(IfColonDelimitedBody),
}

/// Represents the body of an `if` statement when it is a statement body.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct IfStatementBody {
    pub statement: Box<Statement>,
    pub else_if_clauses: Sequence<IfStatementBodyElseIfClause>,
    pub else_clause: Option<IfStatementBodyElseClause>,
}

/// Represents an `elseif` clause in a statement body of an `if` statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct IfStatementBodyElseIfClause {
    pub elseif: Keyword,
    pub left_parenthesis: Span,
    pub condition: Box<Expression>,
    pub right_parenthesis: Span,
    pub statement: Box<Statement>,
}

/// Represents an `else` clause in a statement body of an `if` statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct IfStatementBodyElseClause {
    pub r#else: Keyword,
    pub statement: Box<Statement>,
}

/// Represents a colon-delimited body of an `if` statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct IfColonDelimitedBody {
    pub colon: Span,
    pub statements: Sequence<Statement>,
    pub else_if_clauses: Sequence<IfColonDelimitedBodyElseIfClause>,
    pub else_clause: Option<IfColonDelimitedBodyElseClause>,
    pub endif: Keyword,
    pub terminator: Terminator,
}

/// Represents an `elseif` clause in a colon-delimited body of an `if` statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct IfColonDelimitedBodyElseIfClause {
    pub elseif: Keyword,
    pub left_parenthesis: Span,
    pub condition: Box<Expression>,
    pub right_parenthesis: Span,
    pub colon: Span,
    pub statements: Sequence<Statement>,
}

/// Represents an `else` clause in a colon-delimited body of an `if` statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct IfColonDelimitedBodyElseClause {
    pub r#else: Keyword,
    pub colon: Span,
    pub statements: Sequence<Statement>,
}

impl HasSpan for If {
    fn span(&self) -> Span {
        Span::between(self.r#if.span(), self.body.span())
    }
}

impl HasSpan for IfBody {
    fn span(&self) -> Span {
        match self {
            IfBody::Statement(body) => body.span(),
            IfBody::ColonDelimited(body) => body.span(),
        }
    }
}

impl HasSpan for IfStatementBody {
    fn span(&self) -> Span {
        let span = self.statement.span();

        Span::between(
            span,
            self.else_clause.as_ref().map_or_else(|| self.else_if_clauses.span(span.end), |r#else| r#else.span()),
        )
    }
}

impl HasSpan for IfStatementBodyElseIfClause {
    fn span(&self) -> Span {
        Span::between(self.elseif.span(), self.statement.span())
    }
}

impl HasSpan for IfStatementBodyElseClause {
    fn span(&self) -> Span {
        Span::between(self.r#else.span(), self.statement.span())
    }
}

impl HasSpan for IfColonDelimitedBody {
    fn span(&self) -> Span {
        Span::between(self.colon, self.terminator.span())
    }
}

impl HasSpan for IfColonDelimitedBodyElseIfClause {
    fn span(&self) -> Span {
        Span::between(self.elseif.span(), self.statements.span(self.colon.end))
    }
}

impl HasSpan for IfColonDelimitedBodyElseClause {
    fn span(&self) -> Span {
        Span::between(self.r#else.span(), self.statements.span(self.colon.end))
    }
}
