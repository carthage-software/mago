use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::identifier::parse_identifier;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_optional_implements(stream: &mut TokenStream<'_, '_>) -> Result<Option<Implements>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["implements"]) => Some(Implements {
            implements: utils::expect_any_keyword(stream)?,
            types: {
                let mut types = Vec::new();
                let mut commas = Vec::new();
                loop {
                    types.push(parse_identifier(stream)?);

                    match utils::maybe_peek(stream)?.map(|t| t.kind) {
                        Some(T![","]) => {
                            commas.push(utils::expect_any(stream)?);
                        }
                        _ => break,
                    }
                }

                TokenSeparatedSequence::new(types, commas)
            },
        }),
        _ => None,
    })
}

pub fn parse_optional_extends(stream: &mut TokenStream<'_, '_>) -> Result<Option<Extends>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["extends"]) => Some(Extends {
            extends: utils::expect_any_keyword(stream)?,
            types: {
                let mut types = Vec::new();
                let mut commas = Vec::new();
                loop {
                    types.push(parse_identifier(stream)?);

                    match utils::maybe_peek(stream)?.map(|t| t.kind) {
                        Some(T![","]) => {
                            commas.push(utils::expect_any(stream)?);
                        }
                        _ => break,
                    }
                }
                TokenSeparatedSequence::new(types, commas)
            },
        }),
        _ => None,
    })
}
