use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::internal::block::parse_block;
use crate::parser::internal::identifier::parse_identifier;
use crate::parser::internal::statement::parse_statement;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_namespace<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Namespace<'arena>, ParseError> {
    let namespace = utils::expect_keyword(stream, T!["namespace"])?;
    let name = match utils::peek(stream)?.kind {
        T![";" | "?>" | "{"] => None,
        _ => Some(parse_identifier(stream)?),
    };
    let body = parse_namespace_body(stream)?;

    Ok(Namespace { namespace, name, body })
}

pub fn parse_namespace_body<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<NamespaceBody<'arena>, ParseError> {
    let next = utils::peek(stream)?;
    match next.kind {
        T!["{"] => Ok(NamespaceBody::BraceDelimited(parse_block(stream)?)),
        _ => Ok(NamespaceBody::Implicit(parse_namespace_implicit_body(stream)?)),
    }
}

pub fn parse_namespace_implicit_body<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<NamespaceImplicitBody<'arena>, ParseError> {
    let terminator = parse_terminator(stream)?;
    let mut statements = stream.new_vec();
    loop {
        let next = utils::maybe_peek(stream)?.map(|t| t.kind);
        if matches!(next, None | Some(T!["namespace"])) {
            break;
        }

        statements.push(parse_statement(stream)?);
    }

    Ok(NamespaceImplicitBody { terminator, statements: Sequence::new(statements) })
}
