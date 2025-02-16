use mago_ast::ast::*;
use mago_ast::sequence::TokenSeparatedSequence;
use mago_token::Precedence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::argument::parse_optional_argument_list;
use crate::internal::expression::parse_expression;
use crate::internal::expression::parse_expression_with_precedence;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_construct<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Construct<'i>, ParseError> {
    let token = utils::peek(stream)?;

    Ok(match token.kind {
        T!["isset"] => Construct::Isset(IssetConstruct {
            isset: utils::expect_keyword(stream, T!["isset"])?,
            left_parenthesis: utils::expect_span(stream, T!["("])?,
            values: {
                let mut values = stream.vec();
                let mut commas = stream.vec();
                loop {
                    if matches!(utils::peek(stream)?.kind, T![")"]) {
                        break;
                    }

                    values.push(parse_expression(stream)?);

                    match utils::peek(stream)?.kind {
                        T![","] => {
                            commas.push(utils::expect_any(stream)?);
                        }
                        _ => {
                            break;
                        }
                    }
                }

                TokenSeparatedSequence::new(values, commas)
            },
            right_parenthesis: utils::expect_span(stream, T![")"])?,
        }),
        T!["empty"] => {
            let empty = utils::expect_keyword(stream, T!["empty"])?;
            let left_parenthesis = utils::expect_span(stream, T!["("])?;
            let value = parse_expression(stream)?;
            let right_parenthesis = utils::expect_span(stream, T![")"])?;

            Construct::Empty(EmptyConstruct { empty, left_parenthesis, value: stream.boxed(value), right_parenthesis })
        }
        T!["eval"] => {
            let eval = utils::expect_keyword(stream, T!["eval"])?;
            let left_parenthesis = utils::expect_span(stream, T!["("])?;
            let value = parse_expression(stream)?;
            let right_parenthesis = utils::expect_span(stream, T![")"])?;

            Construct::Eval(EvalConstruct { eval, left_parenthesis, value: stream.boxed(value), right_parenthesis })
        }
        T!["print"] => {
            let print = utils::expect_any_keyword(stream)?;
            let value = parse_expression_with_precedence(stream, Precedence::Print)?;

            Construct::Print(PrintConstruct { print, value: stream.boxed(value) })
        }
        T!["require"] => {
            let require = utils::expect_any_keyword(stream)?;
            let value = parse_expression(stream)?;

            Construct::Require(RequireConstruct { require, value: stream.boxed(value) })
        }
        T!["require_once"] => {
            let require_once = utils::expect_any_keyword(stream)?;
            let value = parse_expression(stream)?;

            Construct::RequireOnce(RequireOnceConstruct { require_once, value: stream.boxed(value) })
        }
        T!["include"] => {
            let include = utils::expect_any_keyword(stream)?;
            let value = parse_expression(stream)?;

            Construct::Include(IncludeConstruct { include, value: stream.boxed(value) })
        }
        T!["include_once"] => {
            let include_once = utils::expect_any_keyword(stream)?;
            let value = parse_expression(stream)?;

            Construct::IncludeOnce(IncludeOnceConstruct { include_once, value: stream.boxed(value) })
        }
        T!["exit"] => Construct::Exit(ExitConstruct {
            exit: utils::expect_any_keyword(stream)?,
            arguments: parse_optional_argument_list(stream)?,
        }),
        T!["die"] => Construct::Die(DieConstruct {
            die: utils::expect_any_keyword(stream)?,
            arguments: parse_optional_argument_list(stream)?,
        }),
        _ => {
            return Err(utils::unexpected(
                stream,
                Some(token),
                T![
                    "isset",
                    "empty",
                    "eval",
                    "include",
                    "include_once",
                    "require",
                    "require_once",
                    "print",
                    "exit",
                    "die"
                ],
            ))
        }
    })
}
