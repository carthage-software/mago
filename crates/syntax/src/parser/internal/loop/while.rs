use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::statement::parse_statement;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_while<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<While<'arena>, ParseError> {
    Ok(While {
        r#while: utils::expect_keyword(stream, T!["while"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        condition: {
            let expression = parse_expression(stream)?;

            stream.alloc(expression)
        },
        right_parenthesis: utils::expect_span(stream, T![")"])?,
        body: parse_while_body(stream)?,
    })
}

pub fn parse_while_body<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<WhileBody<'arena>, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T![":"] => WhileBody::ColonDelimited(parse_while_colon_delimited_body(stream)?),
        _ => WhileBody::Statement({
            let statement = parse_statement(stream)?;

            stream.alloc(statement)
        }),
    })
}

pub fn parse_while_colon_delimited_body<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<WhileColonDelimitedBody<'arena>, ParseError> {
    Ok(WhileColonDelimitedBody {
        colon: utils::expect_span(stream, T![":"])?,
        statements: {
            let mut statements = stream.new_vec();
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
