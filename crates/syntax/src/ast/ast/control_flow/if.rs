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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct If<'arena> {
    pub r#if: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub condition: &'arena Expression<'arena>,
    pub right_parenthesis: Span,
    pub body: IfBody<'arena>,
}

/// Represents the body of an `if` statement.
///
/// This can be either a statement body or a colon-delimited body.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum IfBody<'arena> {
    Statement(IfStatementBody<'arena>),
    ColonDelimited(IfColonDelimitedBody<'arena>),
}

/// Represents the body of an `if` statement when it is a statement body.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IfStatementBody<'arena> {
    pub statement: &'arena Statement<'arena>,
    pub else_if_clauses: Sequence<'arena, IfStatementBodyElseIfClause<'arena>>,
    pub else_clause: Option<IfStatementBodyElseClause<'arena>>,
}

/// Represents an `elseif` clause in a statement body of an `if` statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IfStatementBodyElseIfClause<'arena> {
    pub elseif: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub condition: &'arena Expression<'arena>,
    pub right_parenthesis: Span,
    pub statement: &'arena Statement<'arena>,
}

/// Represents an `else` clause in a statement body of an `if` statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IfStatementBodyElseClause<'arena> {
    pub r#else: Keyword<'arena>,
    pub statement: &'arena Statement<'arena>,
}

/// Represents a colon-delimited body of an `if` statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IfColonDelimitedBody<'arena> {
    pub colon: Span,
    pub statements: Sequence<'arena, Statement<'arena>>,
    pub else_if_clauses: Sequence<'arena, IfColonDelimitedBodyElseIfClause<'arena>>,
    pub else_clause: Option<IfColonDelimitedBodyElseClause<'arena>>,
    pub endif: Keyword<'arena>,
    pub terminator: Terminator<'arena>,
}

/// Represents an `elseif` clause in a colon-delimited body of an `if` statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IfColonDelimitedBodyElseIfClause<'arena> {
    pub elseif: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub condition: &'arena Expression<'arena>,
    pub right_parenthesis: Span,
    pub colon: Span,
    pub statements: Sequence<'arena, Statement<'arena>>,
}

/// Represents an `else` clause in a colon-delimited body of an `if` statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IfColonDelimitedBodyElseClause<'arena> {
    pub r#else: Keyword<'arena>,
    pub colon: Span,
    pub statements: Sequence<'arena, Statement<'arena>>,
}

impl<'arena> IfBody<'arena> {
    pub const fn has_else_clause(&self) -> bool {
        match &self {
            IfBody::Statement(if_statement_body) => if_statement_body.else_clause.is_some(),
            IfBody::ColonDelimited(if_colon_delimited_body) => if_colon_delimited_body.else_clause.is_some(),
        }
    }

    pub fn has_else_if_clauses(&self) -> bool {
        match &self {
            IfBody::Statement(if_statement_body) => !if_statement_body.else_if_clauses.is_empty(),
            IfBody::ColonDelimited(if_colon_delimited_body) => !if_colon_delimited_body.else_if_clauses.is_empty(),
        }
    }

    pub fn statements(&self) -> &[Statement<'arena>] {
        match &self {
            IfBody::Statement(if_statement_body) => std::slice::from_ref(if_statement_body.statement),
            IfBody::ColonDelimited(if_colon_delimited_body) => if_colon_delimited_body.statements.as_slice(),
        }
    }

    pub fn else_statements(&self) -> Option<&[Statement<'arena>]> {
        match &self {
            IfBody::Statement(if_statement_body) => {
                if_statement_body.else_clause.as_ref().map(|e| std::slice::from_ref(e.statement))
            }
            IfBody::ColonDelimited(if_colon_delimited_body) => {
                if_colon_delimited_body.else_clause.as_ref().map(|e| e.statements.as_slice())
            }
        }
    }

    pub fn else_if_statements(&self) -> Vec<&[Statement<'arena>]> {
        match &self {
            IfBody::Statement(if_statement_body) => {
                if_statement_body.else_if_clauses.iter().map(|e| std::slice::from_ref(e.statement)).collect()
            }
            IfBody::ColonDelimited(if_colon_delimited_body) => {
                if_colon_delimited_body.else_if_clauses.iter().map(|e| e.statements.as_slice()).collect()
            }
        }
    }

    pub fn else_if_clauses(&self) -> Vec<(&Expression<'arena>, &[Statement<'arena>])> {
        match &self {
            IfBody::Statement(if_statement_body) => if_statement_body
                .else_if_clauses
                .iter()
                .map(|e| (e.condition, std::slice::from_ref(e.statement)))
                .collect(),
            IfBody::ColonDelimited(if_colon_delimited_body) => {
                if_colon_delimited_body.else_if_clauses.iter().map(|e| (e.condition, e.statements.as_slice())).collect()
            }
        }
    }
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
            self.else_clause
                .as_ref()
                .map_or_else(|| self.else_if_clauses.span(span.file_id, span.end), |r#else| r#else.span()),
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
        Span::between(self.elseif.span(), self.statements.span(self.colon.file_id, self.colon.end))
    }
}

impl HasSpan for IfColonDelimitedBodyElseClause<'_> {
    fn span(&self) -> Span {
        Span::between(self.r#else.span(), self.statements.span(self.colon.file_id, self.colon.end))
    }
}
