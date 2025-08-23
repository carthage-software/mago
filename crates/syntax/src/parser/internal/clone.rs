use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::argument::parse_argument;
use crate::parser::internal::argument::parse_remaining_argument_list;
use crate::parser::internal::expression::parse_expression_with_precedence;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;
use crate::token::Precedence;
use crate::token::TokenKind;

/// Parses a `clone` expression, handling the syntactic ambiguity introduced in PHP 8.5.
///
/// PHP 8.5 allows `clone` to be used like a function (e.g., `clone($foo, $bar)`). This
/// creates an ambiguity with the older syntax `clone ($foo)`, which should be parsed as
/// a `clone` expression operating on a parenthesized expression, not a function call.
///
/// This function resolves the ambiguity by looking ahead after the first argument. If the
/// next token is not a comma and the argument is a simple positional one, it assumes
/// the legacy `clone (expr)` structure. Otherwise, it parses the expression as a
/// standard function call.
pub fn parse_ambiguous_clone_expression<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Expression<'arena>, ParseError> {
    let clone = utils::expect_keyword(stream, T!["clone"])?;
    if utils::peek(stream)?.kind != TokenKind::LeftParenthesis {
        return Ok(Expression::Clone(Clone {
            clone,
            object: {
                let object = parse_expression_with_precedence(stream, Precedence::Clone)?;

                stream.alloc(object)
            },
        }));
    }

    let left_parenthesis = utils::expect_span(stream, T!["("])?;
    if let TokenKind::DotDotDot = utils::peek(stream)?.kind {
        let ellipsis = utils::expect_any(stream)?.span;
        let right_parenthesis = utils::expect_span(stream, T![")"])?;

        return Ok(Expression::ClosureCreation(ClosureCreation::Function(FunctionClosureCreation {
            function: stream.alloc(Expression::Identifier(Identifier::Local(LocalIdentifier {
                span: clone.span,
                value: clone.value,
            }))),
            left_parenthesis,
            ellipsis,
            right_parenthesis,
        })));
    }

    let argument = parse_argument(stream)?;
    let is_next_comma = utils::peek(stream)?.kind.is_comma();

    let cloned_expression = match argument {
        Argument::Positional(argument) if !is_next_comma && argument.ellipsis.is_none() => argument.value,
        _ => {
            let argument_list = parse_remaining_argument_list(stream, left_parenthesis, argument)?;

            return Ok(Expression::Call(Call::Function(FunctionCall {
                function: stream.alloc(Expression::Identifier(Identifier::Local(LocalIdentifier {
                    span: clone.span,
                    value: clone.value,
                }))),
                argument_list,
            })));
        }
    };

    Ok(Expression::Clone(Clone {
        clone,
        object: {
            let object = Expression::Parenthesized(Parenthesized {
                left_parenthesis,
                expression: stream.alloc(cloned_expression),
                right_parenthesis: utils::expect_span(stream, T![")"])?,
            });

            stream.alloc(object)
        },
    }))
}
