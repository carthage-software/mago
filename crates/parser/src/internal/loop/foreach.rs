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
pub fn parse_foreach<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Foreach<'i>, ParseError> {
    let foreach = utils::expect_keyword(stream, T!["foreach"])?;
    let left_parenthesis = utils::expect_span(stream, T!["("])?;
    let expression = parse_expression(stream)?;
    let r#as = utils::expect_keyword(stream, T!["as"])?;
    let target = parse_foreach_target(stream)?;
    let right_parenthesis = utils::expect_span(stream, T![")"])?;
    let body = parse_foreach_body(stream)?;

    Ok(Foreach {
        foreach,
        left_parenthesis,
        expression: stream.boxed(expression),
        r#as,
        target,
        right_parenthesis,
        body,
    })
}

#[inline]
pub fn parse_foreach_target<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<ForeachTarget<'i>, ParseError> {
    let key_or_value = parse_expression(stream)?;

    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["=>"]) => {
            let key = key_or_value;
            let double_arrow = utils::expect_span(stream, T!["=>"])?;
            let value = parse_expression(stream)?;

            ForeachTarget::KeyValue(ForeachKeyValueTarget {
                key: stream.boxed(key),
                double_arrow,
                value: stream.boxed(value),
            })
        }
        _ => ForeachTarget::Value(ForeachValueTarget { value: stream.boxed(key_or_value) }),
    })
}

#[inline]
pub fn parse_foreach_body<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<ForeachBody<'i>, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T![":"] => ForeachBody::ColonDelimited(parse_foreach_colon_delimited_body(stream)?),
        _ => {
            let statement = parse_statement(stream)?;

            ForeachBody::Statement(stream.boxed(statement))
        }
    })
}

#[inline]
pub fn parse_foreach_colon_delimited_body<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<ForeachColonDelimitedBody<'i>, ParseError> {
    Ok(ForeachColonDelimitedBody {
        colon: utils::expect_span(stream, T![":"])?,
        statements: {
            let mut statements = stream.vec();
            loop {
                if matches!(utils::peek(stream)?.kind, T!["endforeach"]) {
                    break;
                }

                statements.push(parse_statement(stream)?);
            }

            Sequence::new(statements)
        },
        end_foreach: utils::expect_keyword(stream, T!["endforeach"])?,
        terminator: parse_terminator(stream)?,
    })
}
