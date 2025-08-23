use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::statement::parse_statement;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_do_while<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<DoWhile<'arena>, ParseError> {
    Ok(DoWhile {
        r#do: utils::expect_keyword(stream, T!["do"])?,
        statement: {
            let inner = parse_statement(stream)?;

            stream.alloc(inner)
        },
        r#while: utils::expect_keyword(stream, T!["while"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        condition: {
            let inner = parse_expression(stream)?;

            stream.alloc(inner)
        },
        right_parenthesis: utils::expect_span(stream, T![")"])?,
        terminator: parse_terminator(stream)?,
    })
}
