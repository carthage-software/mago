use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::statement::parse_statement;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_if<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<If<'i>, ParseError> {
    let r#if = utils::expect_keyword(stream, T!["if"])?;
    let left_parenthesis = utils::expect_span(stream, T!["("])?;
    let condition = parse_expression(stream)?;
    let right_parenthesis = utils::expect_span(stream, T![")"])?;
    let body = parse_if_body(stream)?;

    Ok(If { r#if, left_parenthesis, condition: stream.boxed(condition), right_parenthesis, body })
}

#[inline]
pub fn parse_if_body<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<IfBody<'i>, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T![":"] => IfBody::ColonDelimited(parse_if_colon_delimited_body(stream)?),
        _ => IfBody::Statement(parse_if_statement_body(stream)?),
    })
}

#[inline]
pub fn parse_if_statement_body<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<IfStatementBody<'i>, ParseError> {
    let statement = parse_statement(stream)?;

    Ok(IfStatementBody {
        statement: stream.boxed(statement),
        else_if_clauses: {
            let mut else_if_clauses = stream.vec();
            while let Some(else_if_clause) = parse_optional_if_statement_body_else_if_clause(stream)? {
                else_if_clauses.push(else_if_clause);
            }

            Sequence::new(else_if_clauses)
        },
        else_clause: parse_optional_if_statement_body_else_clause(stream)?,
    })
}

#[inline]
pub fn parse_optional_if_statement_body_else_if_clause<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<Option<IfStatementBodyElseIfClause<'i>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["elseif"]) => Some(parse_if_statement_body_else_if_clause(stream)?),
        _ => None,
    })
}

#[inline]
pub fn parse_if_statement_body_else_if_clause<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<IfStatementBodyElseIfClause<'i>, ParseError> {
    let elseif = utils::expect_keyword(stream, T!["elseif"])?;
    let left_parenthesis = utils::expect_span(stream, T!["("])?;
    let condition = parse_expression(stream)?;
    let right_parenthesis = utils::expect_span(stream, T![")"])?;
    let statement = parse_statement(stream)?;

    Ok(IfStatementBodyElseIfClause {
        elseif,
        left_parenthesis,
        condition: stream.boxed(condition),
        right_parenthesis,
        statement: stream.boxed(statement),
    })
}

#[inline]
pub fn parse_optional_if_statement_body_else_clause<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<Option<IfStatementBodyElseClause<'i>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["else"]) => Some(parse_if_statement_body_else_clause(stream)?),
        _ => None,
    })
}

#[inline]
pub fn parse_if_statement_body_else_clause<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<IfStatementBodyElseClause<'i>, ParseError> {
    let r#else = utils::expect_keyword(stream, T!["else"])?;
    let statement = parse_statement(stream)?;

    Ok(IfStatementBodyElseClause { r#else, statement: stream.boxed(statement) })
}

#[inline]
pub fn parse_if_colon_delimited_body<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<IfColonDelimitedBody<'i>, ParseError> {
    Ok(IfColonDelimitedBody {
        colon: utils::expect_span(stream, T![":"])?,
        statements: {
            let mut statements = stream.vec();
            loop {
                if matches!(utils::peek(stream)?.kind, T!["elseif" | "else" | "endif"]) {
                    break;
                }

                statements.push(parse_statement(stream)?);
            }

            Sequence::new(statements)
        },
        else_if_clauses: {
            let mut else_if_clauses = stream.vec();
            while let Some(else_if_clause) = parse_optional_if_colon_delimited_body_else_if_clause(stream)? {
                else_if_clauses.push(else_if_clause);
            }

            Sequence::new(else_if_clauses)
        },
        else_clause: parse_optional_if_colon_delimited_body_else_clause(stream)?,
        endif: utils::expect_keyword(stream, T!["endif"])?,
        terminator: parse_terminator(stream)?,
    })
}

#[inline]
pub fn parse_optional_if_colon_delimited_body_else_if_clause<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<Option<IfColonDelimitedBodyElseIfClause<'i>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["elseif"]) => Some(parse_if_colon_delimited_body_else_if_clause(stream)?),
        _ => None,
    })
}

#[inline]
pub fn parse_if_colon_delimited_body_else_if_clause<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<IfColonDelimitedBodyElseIfClause<'i>, ParseError> {
    let r#elseif = utils::expect_keyword(stream, T!["elseif"])?;
    let left_parenthesis = utils::expect_span(stream, T!["("])?;
    let condition = parse_expression(stream)?;
    let right_parenthesis = utils::expect_span(stream, T![")"])?;
    let colon = utils::expect_span(stream, T![":"])?;
    let statements = {
        let mut statements = stream.vec();
        loop {
            if matches!(utils::peek(stream)?.kind, T!["elseif" | "else" | "endif"]) {
                break;
            }

            statements.push(parse_statement(stream)?);
        }

        Sequence::new(statements)
    };

    Ok(IfColonDelimitedBodyElseIfClause {
        r#elseif,
        left_parenthesis,
        condition: stream.boxed(condition),
        right_parenthesis,
        colon,
        statements,
    })
}

#[inline]
pub fn parse_optional_if_colon_delimited_body_else_clause<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<Option<IfColonDelimitedBodyElseClause<'i>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["else"]) => Some(parse_if_colon_delimited_body_else_clause(stream)?),
        _ => None,
    })
}

#[inline]
pub fn parse_if_colon_delimited_body_else_clause<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<IfColonDelimitedBodyElseClause<'i>, ParseError> {
    Ok(IfColonDelimitedBodyElseClause {
        r#else: utils::expect_keyword(stream, T!["else"])?,
        colon: utils::expect_span(stream, T![":"])?,
        statements: {
            let mut statements = stream.vec();
            loop {
                if matches!(utils::peek(stream)?.kind, T!["endif"]) {
                    break;
                }

                statements.push(parse_statement(stream)?);
            }
            Sequence::new(statements)
        },
    })
}
