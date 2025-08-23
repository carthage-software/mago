use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_identifier<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Identifier<'arena>, ParseError> {
    let token = utils::peek(stream)?;

    Ok(match &token.kind {
        T![QualifiedIdentifier] => Identifier::Qualified(parse_qualified_identifier(stream)?),
        T![FullyQualifiedIdentifier] => Identifier::FullyQualified(parse_fully_qualified_identifier(stream)?),
        _ => Identifier::Local(parse_local_identifier(stream)?),
    })
}

pub fn parse_local_identifier<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<LocalIdentifier<'arena>, ParseError> {
    let token = utils::expect_any(stream)?;

    if !token.kind.is_identifier_maybe_reserved() {
        return Err(utils::unexpected(stream, Some(token), &[T![Identifier]]));
    }

    Ok(LocalIdentifier { span: token.span, value: token.value })
}

pub fn parse_qualified_identifier<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<QualifiedIdentifier<'arena>, ParseError> {
    let token = utils::expect(stream, T![QualifiedIdentifier])?;

    Ok(QualifiedIdentifier { span: token.span, value: token.value })
}

pub fn parse_fully_qualified_identifier<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<FullyQualifiedIdentifier<'arena>, ParseError> {
    let token = utils::expect(stream, T![FullyQualifiedIdentifier])?;

    Ok(FullyQualifiedIdentifier { span: token.span, value: token.value })
}
