use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::identifier::parse_identifier;
use crate::internal::statement::parse_statement;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

use super::block::parse_block;

#[inline]
pub fn parse_namespace<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Namespace<'i>, ParseError> {
    let namespace = utils::expect_keyword(stream, T!["namespace"])?;
    let name = match utils::peek(stream)?.kind {
        T![";" | "?>" | "{"] => None,
        _ => Some(parse_identifier(stream)?),
    };
    let body = parse_namespace_body(stream)?;

    Ok(Namespace { namespace, name, body })
}

#[inline]
pub fn parse_namespace_body<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<NamespaceBody<'i>, ParseError> {
    let next = utils::peek(stream)?;
    match next.kind {
        T!["{"] => Ok(NamespaceBody::BraceDelimited(parse_block(stream)?)),
        _ => Ok(NamespaceBody::Implicit(parse_namespace_implicit_body(stream)?)),
    }
}

#[inline]
pub fn parse_namespace_implicit_body<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<NamespaceImplicitBody<'i>, ParseError> {
    let terminator = parse_terminator(stream)?;
    let mut statements = stream.vec();
    loop {
        let next = utils::maybe_peek(stream)?.map(|t| t.kind);
        if matches!(next, None | Some(T!["namespace"])) {
            break;
        }

        statements.push(parse_statement(stream)?);
    }

    Ok(NamespaceImplicitBody { terminator, statements: Sequence::new(statements) })
}
