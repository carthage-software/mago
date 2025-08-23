use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_modifier_sequence<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Sequence<'arena, Modifier<'arena>>, ParseError> {
    let mut modifiers = stream.new_vec();
    while let Some(modifier) = parse_optional_modifier(stream)? {
        modifiers.push(modifier);
    }

    Ok(Sequence::new(modifiers))
}

pub fn parse_optional_read_visibility_modifier<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<Modifier<'arena>>, ParseError> {
    Ok(Some(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["public"]) => Modifier::Public(utils::expect_any_keyword(stream)?),
        Some(T!["protected"]) => Modifier::Protected(utils::expect_any_keyword(stream)?),
        Some(T!["private"]) => Modifier::Private(utils::expect_any_keyword(stream)?),
        _ => return Ok(None),
    }))
}

pub fn parse_optional_modifier<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<Modifier<'arena>>, ParseError> {
    Ok(Some(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["public"]) => Modifier::Public(utils::expect_any_keyword(stream)?),
        Some(T!["protected"]) => Modifier::Protected(utils::expect_any_keyword(stream)?),
        Some(T!["private"]) => Modifier::Private(utils::expect_any_keyword(stream)?),
        Some(T!["static"]) => Modifier::Static(utils::expect_any_keyword(stream)?),
        Some(T!["final"]) => Modifier::Final(utils::expect_any_keyword(stream)?),
        Some(T!["abstract"]) => Modifier::Abstract(utils::expect_any_keyword(stream)?),
        Some(T!["readonly"]) => Modifier::Readonly(utils::expect_any_keyword(stream)?),
        Some(T!["private(set)"]) => Modifier::PrivateSet(utils::expect_any_keyword(stream)?),
        Some(T!["protected(set)"]) => Modifier::ProtectedSet(utils::expect_any_keyword(stream)?),
        Some(T!["public(set)"]) => Modifier::PublicSet(utils::expect_any_keyword(stream)?),
        _ => return Ok(None),
    }))
}
