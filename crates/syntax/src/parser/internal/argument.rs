use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::expression;
use crate::parser::internal::identifier;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_optional_argument_list<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<ArgumentList<'arena>>, ParseError> {
    if utils::peek(stream)?.kind == T!["("] { Ok(Some(parse_argument_list(stream)?)) } else { Ok(None) }
}

pub fn parse_argument_list<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<ArgumentList<'arena>, ParseError> {
    Ok(ArgumentList {
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        arguments: {
            let mut arguments = stream.new_vec();
            let mut commas = stream.new_vec();
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

pub fn parse_argument<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Argument<'arena>, ParseError> {
    if utils::peek(stream)?.kind.is_identifier_maybe_reserved()
        && matches!(utils::maybe_peek_nth(stream, 1)?.map(|token| token.kind), Some(T![":"]))
    {
        return Ok(Argument::Named(NamedArgument {
            name: identifier::parse_local_identifier(stream)?,
            colon: utils::expect_any(stream)?.span,
            value: expression::parse_expression(stream)?,
        }));
    }

    Ok(Argument::Positional(PositionalArgument {
        ellipsis: utils::maybe_expect(stream, T!["..."])?.map(|token| token.span),
        value: expression::parse_expression(stream)?,
    }))
}

pub fn parse_partial_argument_list<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<PartialArgumentList<'arena>, ParseError> {
    Ok(PartialArgumentList {
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        arguments: {
            let mut arguments = stream.new_vec();
            let mut commas = stream.new_vec();
            loop {
                let next = utils::peek(stream)?;
                if next.kind == T![")"] {
                    break;
                }

                arguments.push(parse_partial_argument(stream)?);

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

pub fn parse_partial_argument<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<PartialArgument<'arena>, ParseError> {
    let current = utils::peek(stream)?;

    if current.kind == T!["?"] {
        return Ok(PartialArgument::Placeholder(Placeholder { span: utils::expect_any(stream)?.span }));
    }

    if current.kind == T!["..."] {
        let next = utils::maybe_peek_nth(stream, 1)?;
        match next.map(|t| t.kind) {
            Some(T![","]) | Some(T![")"]) | None => {
                return Ok(PartialArgument::VariadicPlaceholder(VariadicPlaceholder {
                    span: utils::expect_any(stream)?.span,
                }));
            }
            _ => {}
        }
    }

    if current.kind.is_identifier_maybe_reserved()
        && matches!(utils::maybe_peek_nth(stream, 1)?.map(|token| token.kind), Some(T![":"]))
    {
        let name = identifier::parse_local_identifier(stream)?;
        let colon = utils::expect_any(stream)?.span;

        if utils::peek(stream)?.kind == T!["?"] {
            return Ok(PartialArgument::NamedPlaceholder(NamedPlaceholder {
                name,
                colon,
                question_mark: utils::expect_any(stream)?.span,
            }));
        }

        return Ok(PartialArgument::Named(NamedArgument { name, colon, value: expression::parse_expression(stream)? }));
    }

    Ok(PartialArgument::Positional(PositionalArgument {
        ellipsis: utils::maybe_expect(stream, T!["..."])?.map(|token| token.span),
        value: expression::parse_expression(stream)?,
    }))
}
