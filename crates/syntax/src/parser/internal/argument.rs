use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::expression;
use crate::parser::internal::identifier;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_optional_argument_list(stream: &mut TokenStream<'_, '_>) -> Result<Option<ArgumentList>, ParseError> {
    let next = utils::peek(stream)?;
    if next.kind == T!["("] { Ok(Some(parse_argument_list(stream)?)) } else { Ok(None) }
}

pub fn parse_argument_list(stream: &mut TokenStream<'_, '_>) -> Result<ArgumentList, ParseError> {
    Ok(ArgumentList {
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        arguments: {
            let mut arguments = Vec::new();
            let mut commas = Vec::new();
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

pub fn parse_argument(stream: &mut TokenStream<'_, '_>) -> Result<Argument, ParseError> {
    let token = utils::peek(stream)?;

    if token.kind.is_identifier_maybe_reserved()
        && matches!(utils::maybe_peek_nth(stream, 1)?.map(|token| token.kind), Some(T![":"]))
    {
        return Ok(Argument::Named(NamedArgument {
            name: identifier::parse_local_identifier(stream)?,
            colon: utils::expect(stream, T![":"])?.span,
            ellipsis: utils::maybe_expect(stream, T!["..."])?.map(|token| token.span),
            value: expression::parse_expression(stream)?,
        }));
    }

    Ok(Argument::Positional(PositionalArgument {
        ellipsis: utils::maybe_expect(stream, T!["..."])?.map(|token| token.span),
        value: expression::parse_expression(stream)?,
    }))
}
