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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct If<'a> {
    pub r#if: Keyword,
    pub left_parenthesis: Span,
    pub condition: Box<'a, Expression<'a>>,
    pub right_parenthesis: Span,
    pub body: IfBody<'a>,
}

/// Represents the body of an `if` statement.
///
/// This can be either a statement body or a colon-delimited body.
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum IfBody<'a> {
    Statement(IfStatementBody<'a>),
    ColonDelimited(IfColonDelimitedBody<'a>),
}

/// Represents the body of an `if` statement when it is a statement body.
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct IfStatementBody<'a> {
    pub statement: Box<'a, Statement<'a>>,
    pub else_if_clauses: Sequence<'a, IfStatementBodyElseIfClause<'a>>,
    pub else_clause: Option<IfStatementBodyElseClause<'a>>,
}

/// Represents an `elseif` clause in a statement body of an `if` statement.
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct IfStatementBodyElseIfClause<'a> {
    pub elseif: Keyword,
    pub left_parenthesis: Span,
    pub condition: Box<'a, Expression<'a>>,
    pub right_parenthesis: Span,
    pub statement: Box<'a, Statement<'a>>,
}

/// Represents an `else` clause in a statement body of an `if` statement.
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct IfStatementBodyElseClause<'a> {
    pub r#else: Keyword,
    pub statement: Box<'a, Statement<'a>>,
}

/// Represents a colon-delimited body of an `if` statement.
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct IfColonDelimitedBody<'a> {
    pub colon: Span,
    pub statements: Sequence<'a, Statement<'a>>,
    pub else_if_clauses: Sequence<'a, IfColonDelimitedBodyElseIfClause<'a>>,
    pub else_clause: Option<IfColonDelimitedBodyElseClause<'a>>,
    pub endif: Keyword,
    pub terminator: Terminator,
}

/// Represents an `elseif` clause in a colon-delimited body of an `if` statement.
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct IfColonDelimitedBodyElseIfClause<'a> {
    pub elseif: Keyword,
    pub left_parenthesis: Span,
    pub condition: Box<'a, Expression<'a>>,
    pub right_parenthesis: Span,
    pub colon: Span,
    pub statements: Sequence<'a, Statement<'a>>,
}

/// Represents an `else` clause in a colon-delimited body of an `if` statement.
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct IfColonDelimitedBodyElseClause<'a> {
    pub r#else: Keyword,
    pub colon: Span,
    pub statements: Sequence<'a, Statement<'a>>,
}

impl HasSpan for If<'_> {
    fn span(&self) -> Span {
        Span::between(self.r#if.span(), self.body.span())
    }
}

impl HasSpan for IfBody<'_> {
    fn span(&self) -> Span {
        match self {
            IfBody::Statement(body) => body.span(),
            IfBody::ColonDelimited(body) => body.span(),
        }
    }
}

impl HasSpan for IfStatementBody<'_> {
    fn span(&self) -> Span {
        let span = self.statement.span();

        Span::between(
            span,
            self.else_clause.as_ref().map_or_else(|| self.else_if_clauses.span(span.end), |r#else| r#else.span()),
        )
    }
}

impl HasSpan for IfStatementBodyElseIfClause<'_> {
    fn span(&self) -> Span {
        Span::between(self.elseif.span(), self.statement.span())
    }
}

impl HasSpan for IfStatementBodyElseClause<'_> {
    fn span(&self) -> Span {
        Span::between(self.r#else.span(), self.statement.span())
    }
}

impl HasSpan for IfColonDelimitedBody<'_> {
    fn span(&self) -> Span {
        Span::between(self.colon, self.terminator.span())
    }
}

impl HasSpan for IfColonDelimitedBodyElseIfClause<'_> {
    fn span(&self) -> Span {
        Span::between(self.elseif.span(), self.statements.span(self.colon.end))
    }
}

impl HasSpan for IfColonDelimitedBodyElseClause<'_> {
    fn span(&self) -> Span {
        Span::between(self.r#else.span(), self.statements.span(self.colon.end))
    }
}
