use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::statement::parse_statement;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

pub fn parse_while(stream: &mut TokenStream<'_, '_>) -> Result<While, ParseError> {
    Ok(While {
        r#while: utils::expect_keyword(stream, T!["while"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        condition: Box::new(parse_expression(stream)?),
        right_parenthesis: utils::expect_span(stream, T![")"])?,
        body: parse_while_body(stream)?,
    })
}

pub fn parse_while_body(stream: &mut TokenStream<'_, '_>) -> Result<WhileBody, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T![":"] => WhileBody::ColonDelimited(parse_while_colon_delimited_body(stream)?),
        _ => WhileBody::Statement(Box::new(parse_statement(stream)?)),
    })
}

pub fn parse_while_colon_delimited_body(
    stream: &mut TokenStream<'_, '_>,
) -> Result<WhileColonDelimitedBody, ParseError> {
    Ok(WhileColonDelimitedBody {
        colon: utils::expect_span(stream, T![":"])?,
        statements: {
            let mut statements = vec![];
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
