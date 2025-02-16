use mago_ast::ast::*;
use mago_ast::sequence::TokenSeparatedSequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression;
use crate::internal::identifier;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_optional_argument_list<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<Option<ArgumentList<'i>>, ParseError> {
    let next = utils::peek(stream)?;
    if next.kind == T!["("] {
        Ok(Some(parse_argument_list(stream)?))
    } else {
        Ok(None)
    }
}

#[inline]
pub fn parse_argument_list<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<ArgumentList<'i>, ParseError> {
    Ok(ArgumentList {
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        arguments: {
            let mut arguments = stream.vec();
            let mut commas = stream.vec();
            loop {
                let next = utils::peek(stream)?;
                if next.kind == T![")"] {
                    break;
                }

                arguments.push(parse_argument(stream)?);

                let next = utils::peek(stream)?;
                if next.kind == T![","] {
                    commas.push(utils::expect_any(stream)?);
                } else {
                    break;
                }
            }

            TokenSeparatedSequence::new(arguments, commas)
        },
        right_parenthesis: utils::expect_span(stream, T![")"])?,
    })
}

#[inline]
pub fn parse_argument<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Argument<'i>, ParseError> {
    let token = utils::peek(stream)?;

    if token.kind.is_identifier_maybe_reserved()
        && matches!(utils::maybe_peek_nth(stream, 1)?.map(|token| token.kind), Some(T![":"]))
    {
        let name = identifier::parse_local_identifier(stream)?;
        let colon = utils::expect(stream, T![":"])?.span;
        let ellipsis = utils::maybe_expect(stream, T!["..."])?.map(|token| token.span);
        let value = expression::parse_expression(stream)?;

        return Ok(Argument::Named(NamedArgument { name, colon, ellipsis, value: stream.boxed(value) }));
    }

    let ellipsis = utils::maybe_expect(stream, T!["..."])?.map(|token| token.span);
    let value = expression::parse_expression(stream)?;

    Ok(Argument::Positional(PositionalArgument { ellipsis, value: stream.boxed(value) }))
}
