use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::statement::parse_statement;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_if<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<If<'arena>, ParseError> {
    Ok(If {
        r#if: utils::expect_keyword(stream, T!["if"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        condition: {
            let expression = parse_expression(stream)?;

            stream.alloc(expression)
        },
        right_parenthesis: utils::expect_span(stream, T![")"])?,
        body: parse_if_body(stream)?,
    })
}

pub fn parse_if_body<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<IfBody<'arena>, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T![":"] => IfBody::ColonDelimited(parse_if_colon_delimited_body(stream)?),
        _ => IfBody::Statement(parse_if_statement_body(stream)?),
    })
}

pub fn parse_if_statement_body<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<IfStatementBody<'arena>, ParseError> {
    Ok(IfStatementBody {
        statement: {
            let statement = parse_statement(stream)?;

            stream.alloc(statement)
        },
        else_if_clauses: {
            let mut else_if_clauses = stream.new_vec();
            while let Some(else_if_clause) = parse_optional_if_statement_body_else_if_clause(stream)? {
                else_if_clauses.push(else_if_clause);
            }

            Sequence::new(else_if_clauses)
        },
        else_clause: parse_optional_if_statement_body_else_clause(stream)?,
    })
}

pub fn parse_optional_if_statement_body_else_if_clause<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<IfStatementBodyElseIfClause<'arena>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["elseif"]) => Some(parse_if_statement_body_else_if_clause(stream)?),
        _ => None,
    })
}

pub fn parse_if_statement_body_else_if_clause<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<IfStatementBodyElseIfClause<'arena>, ParseError> {
    Ok(IfStatementBodyElseIfClause {
        elseif: utils::expect_keyword(stream, T!["elseif"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        condition: {
            let expression = parse_expression(stream)?;

            stream.alloc(expression)
        },
        right_parenthesis: utils::expect_span(stream, T![")"])?,
        statement: {
            let statement = parse_statement(stream)?;

            stream.alloc(statement)
        },
    })
}

pub fn parse_optional_if_statement_body_else_clause<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<IfStatementBodyElseClause<'arena>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["else"]) => Some(parse_if_statement_body_else_clause(stream)?),
        _ => None,
    })
}

pub fn parse_if_statement_body_else_clause<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<IfStatementBodyElseClause<'arena>, ParseError> {
    Ok(IfStatementBodyElseClause {
        r#else: utils::expect_keyword(stream, T!["else"])?,
        statement: {
            let statement = parse_statement(stream)?;

            stream.alloc(statement)
        },
    })
}

pub fn parse_if_colon_delimited_body<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<IfColonDelimitedBody<'arena>, ParseError> {
    Ok(IfColonDelimitedBody {
        colon: utils::expect_span(stream, T![":"])?,
        statements: {
            let mut statements = stream.new_vec();
            loop {
                if matches!(utils::peek(stream)?.kind, T!["elseif" | "else" | "endif"]) {
                    break;
                }

                statements.push(parse_statement(stream)?);
            }

            Sequence::new(statements)
        },
        else_if_clauses: {
            let mut else_if_clauses = stream.new_vec();
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

pub fn parse_optional_if_colon_delimited_body_else_if_clause<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<IfColonDelimitedBodyElseIfClause<'arena>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["elseif"]) => Some(parse_if_colon_delimited_body_else_if_clause(stream)?),
        _ => None,
    })
}

pub fn parse_if_colon_delimited_body_else_if_clause<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<IfColonDelimitedBodyElseIfClause<'arena>, ParseError> {
    Ok(IfColonDelimitedBodyElseIfClause {
        r#elseif: utils::expect_keyword(stream, T!["elseif"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        condition: {
            let expression = parse_expression(stream)?;

            stream.alloc(expression)
        },
        right_parenthesis: utils::expect_span(stream, T![")"])?,
        colon: utils::expect_span(stream, T![":"])?,
        statements: {
            let mut statements = stream.new_vec();
            loop {
                if matches!(utils::peek(stream)?.kind, T!["elseif" | "else" | "endif"]) {
                    break;
                }

                statements.push(parse_statement(stream)?);
            }

            Sequence::new(statements)
        },
    })
}

pub fn parse_optional_if_colon_delimited_body_else_clause<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<IfColonDelimitedBodyElseClause<'arena>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["else"]) => Some(parse_if_colon_delimited_body_else_clause(stream)?),
        _ => None,
    })
}

pub fn parse_if_colon_delimited_body_else_clause<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<IfColonDelimitedBodyElseClause<'arena>, ParseError> {
    Ok(IfColonDelimitedBodyElseClause {
        r#else: utils::expect_keyword(stream, T!["else"])?,
        colon: utils::expect_span(stream, T![":"])?,
        statements: {
            let mut statements = stream.new_vec();
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
