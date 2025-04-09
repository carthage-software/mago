use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_unset(stream: &mut TokenStream<'_, '_>) -> Result<Unset, ParseError> {
    let unset = utils::expect_keyword(stream, T!["unset"])?;
    let left_parenthesis = utils::expect_span(stream, T!["("])?;

    let mut values = vec![];
    let mut commas = vec![];
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

    let right_parenthesis = utils::expect_span(stream, T![")"])?;
    let terminator = parse_terminator(stream)?;

    Ok(Unset {
        unset,
        left_parenthesis,
        values: TokenSeparatedSequence::new(values, commas),
        right_parenthesis,
        terminator,
    })
}
