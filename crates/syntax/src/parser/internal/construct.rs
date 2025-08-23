use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::argument::parse_optional_argument_list;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::expression::parse_expression_with_precedence;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;
use crate::token::Precedence;

pub fn parse_construct<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Construct<'arena>, ParseError> {
    let token = utils::peek(stream)?;

    Ok(match token.kind {
        T!["isset"] => Construct::Isset(IssetConstruct {
            isset: utils::expect_keyword(stream, T!["isset"])?,
            left_parenthesis: utils::expect_span(stream, T!["("])?,
            values: {
                let mut values = stream.new_vec();
                let mut commas = stream.new_vec();
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
        T!["empty"] => Construct::Empty(EmptyConstruct {
            empty: utils::expect_keyword(stream, T!["empty"])?,
            left_parenthesis: utils::expect_span(stream, T!["("])?,
            value: {
                let expression = parse_expression(stream)?;

                stream.alloc(expression)
            },
            right_parenthesis: utils::expect_span(stream, T![")"])?,
        }),
        T!["eval"] => Construct::Eval(EvalConstruct {
            eval: utils::expect_keyword(stream, T!["eval"])?,
            left_parenthesis: utils::expect_span(stream, T!["("])?,
            value: {
                let expression = parse_expression(stream)?;

                stream.alloc(expression)
            },
            right_parenthesis: utils::expect_span(stream, T![")"])?,
        }),
        T!["print"] => Construct::Print(PrintConstruct {
            print: utils::expect_keyword(stream, T!["print"])?,
            value: {
                let expression = parse_expression_with_precedence(stream, Precedence::Print)?;

                stream.alloc(expression)
            },
        }),
        T!["require"] => Construct::Require(RequireConstruct {
            require: utils::expect_any_keyword(stream)?,
            value: {
                let expression = parse_expression(stream)?;

                stream.alloc(expression)
            },
        }),
        T!["require_once"] => Construct::RequireOnce(RequireOnceConstruct {
            require_once: utils::expect_any_keyword(stream)?,
            value: {
                let expression = parse_expression(stream)?;

                stream.alloc(expression)
            },
        }),
        T!["include"] => Construct::Include(IncludeConstruct {
            include: utils::expect_any_keyword(stream)?,
            value: {
                let expression = parse_expression(stream)?;

                stream.alloc(expression)
            },
        }),
        T!["include_once"] => Construct::IncludeOnce(IncludeOnceConstruct {
            include_once: utils::expect_any_keyword(stream)?,
            value: {
                let expression = parse_expression(stream)?;

                stream.alloc(expression)
            },
        }),
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
            ));
        }
    })
}
