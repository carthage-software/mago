use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::internal::statement::parse_statement;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_block(stream: &mut TokenStream<'_, '_>) -> Result<Block, ParseError> {
    Ok(Block {
        left_brace: utils::expect_span(stream, T!["{"])?,
        statements: {
            let mut statements = Vec::new();
            loop {
                let next = utils::peek(stream)?;
                if matches!(next.kind, T!["}"]) {
                    break;
                }

                statements.push(parse_statement(stream)?);
            }

            Sequence::new(statements)
        },
        right_brace: utils::expect_span(stream, T!["}"])?,
    })
}
