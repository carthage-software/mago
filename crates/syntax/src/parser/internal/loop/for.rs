use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::statement::parse_statement;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_for<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<For<'arena>, ParseError> {
    Ok(For {
        r#for: utils::expect_keyword(stream, T!["for"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        initializations: {
            let mut initializations = stream.new_vec();
            let mut commas = stream.new_vec();
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
            let mut conditions = stream.new_vec();
            let mut commas = stream.new_vec();
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
            let mut increments = stream.new_vec();
            let mut commas = stream.new_vec();
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

pub fn parse_for_body<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<ForBody<'arena>, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T![":"] => ForBody::ColonDelimited(parse_for_colon_delimited_body(stream)?),
        _ => ForBody::Statement({
            let stmt = parse_statement(stream)?;

            stream.alloc(stmt)
        }),
    })
}

pub fn parse_for_colon_delimited_body<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<ForColonDelimitedBody<'arena>, ParseError> {
    Ok(ForColonDelimitedBody {
        colon: utils::expect_span(stream, T![":"])?,
        statements: {
            let mut statements = stream.new_vec();
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
