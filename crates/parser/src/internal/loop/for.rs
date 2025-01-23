use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_ast::sequence::TokenSeparatedSequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::statement::parse_statement;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

pub fn parse_for(stream: &mut TokenStream<'_, '_>) -> Result<For, ParseError> {
    Ok(For {
        r#for: utils::expect_keyword(stream, T!["for"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        initializations: {
            let mut initializations = vec![];
            let mut commas = vec![];
            loop {
                if matches!(utils::peek(stream)?.kind, T![";"]) {
                    break;
                }

                initializations.push(parse_expression(stream)?);

                match utils::peek(stream)?.kind {
                    T![","] => {
                        commas.push(utils::expect_any(stream)?);
                    }
                    _ => {
                        break;
                    }
                }
            }

            TokenSeparatedSequence::new(initializations, commas)
        },
        initializations_semicolon: utils::expect_span(stream, T![";"])?,
        conditions: {
            let mut conditions = vec![];
            let mut commas = vec![];
            loop {
                if matches!(utils::peek(stream)?.kind, T![";"]) {
                    break;
                }

                conditions.push(parse_expression(stream)?);

                match utils::peek(stream)?.kind {
                    T![","] => {
                        commas.push(utils::expect_any(stream)?);
                    }
                    _ => {
                        break;
                    }
                }
            }

            TokenSeparatedSequence::new(conditions, commas)
        },
        conditions_semicolon: utils::expect_span(stream, T![";"])?,
        increments: {
            let mut increments = vec![];
            let mut commas = vec![];
            loop {
                if matches!(utils::peek(stream)?.kind, T![")"]) {
                    break;
                }

                increments.push(parse_expression(stream)?);

                match utils::peek(stream)?.kind {
                    T![","] => {
                        commas.push(utils::expect_any(stream)?);
                    }
                    _ => {
                        break;
                    }
                }
            }

            TokenSeparatedSequence::new(increments, commas)
        },
        right_parenthesis: utils::expect_span(stream, T![")"])?,
        body: parse_for_body(stream)?,
    })
}

pub fn parse_for_body(stream: &mut TokenStream<'_, '_>) -> Result<ForBody, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T![":"] => ForBody::ColonDelimited(parse_for_colon_delimited_body(stream)?),
        _ => ForBody::Statement(Box::new(parse_statement(stream)?)),
    })
}

pub fn parse_for_colon_delimited_body(stream: &mut TokenStream<'_, '_>) -> Result<ForColonDelimitedBody, ParseError> {
    Ok(ForColonDelimitedBody {
        colon: utils::expect_span(stream, T![":"])?,
        statements: {
            let mut statements = vec![];
            loop {
                if matches!(utils::peek(stream)?.kind, T!["endfor"]) {
                    break;
                }

                statements.push(parse_statement(stream)?);
            }

            Sequence::new(statements)
        },
        end_for: utils::expect_keyword(stream, T!["endfor"])?,
        terminator: parse_terminator(stream)?,
    })
}
