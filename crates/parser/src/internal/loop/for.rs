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

#[inline]
pub fn parse_for<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<For<'i>, ParseError> {
    Ok(For {
        r#for: utils::expect_keyword(stream, T!["for"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        initializations: {
            let mut initializations = stream.vec();
            let mut commas = stream.vec();
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
            let mut conditions = stream.vec();
            let mut commas = stream.vec();
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
            let mut increments = stream.vec();
            let mut commas = stream.vec();
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

#[inline]
pub fn parse_for_body<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<ForBody<'i>, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T![":"] => ForBody::ColonDelimited(parse_for_colon_delimited_body(stream)?),
        _ => {
            let statement = parse_statement(stream)?;

            ForBody::Statement(stream.boxed(statement))
        }
    })
}

#[inline]
pub fn parse_for_colon_delimited_body<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<ForColonDelimitedBody<'i>, ParseError> {
    Ok(ForColonDelimitedBody {
        colon: utils::expect_span(stream, T![":"])?,
        statements: {
            let mut statements = stream.vec();
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
