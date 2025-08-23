use mago_database::file::HasFileId;

use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::identifier::parse_identifier;
use crate::parser::internal::identifier::parse_local_identifier;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_use<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Use<'arena>, ParseError> {
    Ok(Use {
        r#use: utils::expect_keyword(stream, T!["use"])?,
        items: parse_use_items(stream)?,
        terminator: parse_terminator(stream)?,
    })
}

pub fn parse_use_items<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<UseItems<'arena>, ParseError> {
    let next = utils::peek(stream)?.kind;

    Ok(match next {
        T!["const" | "function"] => match utils::maybe_peek_nth(stream, 2)?.map(|token| token.kind) {
            Some(T!["\\"]) => UseItems::TypedList(parse_typed_use_item_list(stream)?),
            _ => UseItems::TypedSequence(parse_typed_use_item_sequence(stream)?),
        },
        _ => match utils::maybe_peek_nth(stream, 1)?.map(|token| token.kind) {
            Some(T!["\\"]) => UseItems::MixedList(parse_mixed_use_item_list(stream)?),
            _ => UseItems::Sequence(parse_use_item_sequence(stream)?),
        },
    })
}

pub fn parse_use_item_sequence<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<UseItemSequence<'arena>, ParseError> {
    let start = utils::peek(stream)?.span.start;

    let mut items = stream.new_vec();
    let mut commas = stream.new_vec();
    loop {
        items.push(parse_use_item(stream)?);

        match utils::maybe_expect(stream, T![","])? {
            Some(comma) => {
                commas.push(comma);
            }
            None => break,
        }
    }

    Ok(UseItemSequence { file_id: stream.file_id(), start, items: TokenSeparatedSequence::new(items, commas) })
}

pub fn parse_typed_use_item_sequence<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<TypedUseItemSequence<'arena>, ParseError> {
    let r#type = parse_use_type(stream)?;
    let mut items = stream.new_vec();
    let mut commas = stream.new_vec();
    loop {
        items.push(parse_use_item(stream)?);

        match utils::maybe_expect(stream, T![","])? {
            Some(comma) => {
                commas.push(comma);
            }
            None => break,
        }
    }

    Ok(TypedUseItemSequence { r#type, items: TokenSeparatedSequence::new(items, commas) })
}

pub fn parse_typed_use_item_list<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<TypedUseItemList<'arena>, ParseError> {
    let r#type = parse_use_type(stream)?;
    let namespace = parse_identifier(stream)?;
    let namespace_separator = utils::expect_span(stream, T!["\\"])?;
    let left_brace = utils::expect_span(stream, T!["{"])?;
    let mut items = stream.new_vec();
    let mut commas = stream.new_vec();
    loop {
        let next = utils::peek(stream)?;
        if matches!(next.kind, T!["}"]) {
            break;
        }

        items.push(parse_use_item(stream)?);

        match utils::maybe_expect(stream, T![","])? {
            Some(comma) => {
                commas.push(comma);
            }
            None => break,
        }
    }
    let right_brace = utils::expect_span(stream, T!["}"])?;

    Ok(TypedUseItemList {
        r#type,
        namespace,
        namespace_separator,
        left_brace,
        items: TokenSeparatedSequence::new(items, commas),
        right_brace,
    })
}

pub fn parse_mixed_use_item_list<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<MixedUseItemList<'arena>, ParseError> {
    let namespace = parse_identifier(stream)?;
    let namespace_separator = utils::expect_span(stream, T!["\\"])?;
    let left_brace = utils::expect_span(stream, T!["{"])?;
    let mut items = stream.new_vec();
    let mut commas = stream.new_vec();
    loop {
        let next = utils::peek(stream)?;
        if matches!(next.kind, T!["}"]) {
            break;
        }

        items.push(parse_maybe_typed_use_item(stream)?);

        match utils::maybe_expect(stream, T![","])? {
            Some(comma) => {
                commas.push(comma);
            }
            None => break,
        }
    }
    let right_brace = utils::expect_span(stream, T!["}"])?;

    Ok(MixedUseItemList {
        namespace,
        namespace_separator,
        left_brace,
        items: TokenSeparatedSequence::new(items, commas),
        right_brace,
    })
}

pub fn parse_maybe_typed_use_item<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<MaybeTypedUseItem<'arena>, ParseError> {
    Ok(MaybeTypedUseItem { r#type: parse_optional_use_type(stream)?, item: parse_use_item(stream)? })
}

pub fn parse_optional_use_type<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<UseType<'arena>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["function"]) => Some(UseType::Function(utils::expect_any_keyword(stream)?)),
        Some(T!["const"]) => Some(UseType::Const(utils::expect_any_keyword(stream)?)),
        _ => None,
    })
}

pub fn parse_use_type<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<UseType<'arena>, ParseError> {
    let next = utils::peek(stream)?;

    Ok(match next.kind {
        T!["function"] => UseType::Function(utils::expect_any_keyword(stream)?),
        T!["const"] => UseType::Const(utils::expect_any_keyword(stream)?),
        _ => return Err(utils::unexpected(stream, Some(next), T!["function", "const"])),
    })
}

pub fn parse_use_item<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<UseItem<'arena>, ParseError> {
    Ok(UseItem { name: parse_identifier(stream)?, alias: parse_optional_use_item_alias(stream)? })
}

pub fn parse_optional_use_item_alias<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<UseItemAlias<'arena>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["as"]) => Some(parse_use_item_alias(stream)?),
        _ => None,
    })
}

pub fn parse_use_item_alias<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<UseItemAlias<'arena>, ParseError> {
    let r#as = utils::expect_keyword(stream, T!["as"])?;
    let identifier = parse_local_identifier(stream)?;

    Ok(UseItemAlias { r#as, identifier })
}
