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
pub fn parse_while<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<While<'i>, ParseError> {
    let r#while = utils::expect_keyword(stream, T!["while"])?;
    let left_parenthesis = utils::expect_span(stream, T!["("])?;
    let condition = parse_expression(stream)?;
    let right_parenthesis = utils::expect_span(stream, T![")"])?;
    let body = parse_while_body(stream)?;

    Ok(While { r#while, left_parenthesis, condition: stream.boxed(condition), right_parenthesis, body })
}

#[inline]
pub fn parse_while_body<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<WhileBody<'i>, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T![":"] => WhileBody::ColonDelimited(parse_while_colon_delimited_body(stream)?),
        _ => {
            let statement = parse_statement(stream)?;

            WhileBody::Statement(stream.boxed(statement))
        }
    })
}

#[inline]
pub fn parse_while_colon_delimited_body<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<WhileColonDelimitedBody<'i>, ParseError> {
    Ok(WhileColonDelimitedBody {
        colon: utils::expect_span(stream, T![":"])?,
        statements: {
            let mut statements = stream.vec();
            loop {
                if matches!(utils::peek(stream)?.kind, T!["endwhile"]) {
                    break;
                }

                statements.push(parse_statement(stream)?);
            }

            Sequence::new(statements)
        },
        end_while: utils::expect_keyword(stream, T!["endwhile"])?,
        terminator: parse_terminator(stream)?,
    })
}
