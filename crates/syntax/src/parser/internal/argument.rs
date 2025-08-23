use mago_span::Span;

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

/// Parses the remaining arguments in a list after the opening parenthesis and the first argument
/// have already been consumed.
///
/// This is a helper for parsing constructs where an argument list follows an initial expression,
/// allowing the parser to reuse the standard argument list parsing logic without backtracking.
pub fn parse_remaining_argument_list<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    left_parenthesis: Span,
    initial_argument: Argument<'arena>,
) -> Result<ArgumentList<'arena>, ParseError> {
    let mut arguments = stream.new_vec_of(initial_argument);
    let mut commas = stream.new_vec();

    // Loop to parse the rest of the arguments
    loop {
        let next = utils::peek(stream)?;
        if next.kind == T![")"] {
            break;
        }

        if next.kind == T![","] {
            commas.push(utils::expect_any(stream)?);
            // After a comma, another argument might follow, or a closing parenthesis (for trailing commas).
            if utils::peek(stream)?.kind == T![")"] {
                break;
            }
            arguments.push(parse_argument(stream)?);
        } else {
            // If there's no comma, we expect the list to end.
            break;
        }
    }

    Ok(ArgumentList {
        left_parenthesis,
        arguments: TokenSeparatedSequence::new(arguments, commas),
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
