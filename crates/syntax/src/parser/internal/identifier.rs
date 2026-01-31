use crate::T;
use crate::ast::ast::FullyQualifiedIdentifier;
use crate::ast::ast::Identifier;
use crate::ast::ast::LocalIdentifier;
use crate::ast::ast::QualifiedIdentifier;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_identifier(
        &self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Identifier<'arena>, ParseError> {
        let token = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;

        Ok(match &token.kind {
            T![QualifiedIdentifier] => Identifier::Qualified(self.parse_qualified_identifier(stream)?),
            T![FullyQualifiedIdentifier] => Identifier::FullyQualified(self.parse_fully_qualified_identifier(stream)?),
            _ => Identifier::Local(self.parse_local_identifier(stream)?),
        })
    }

    pub(crate) fn parse_local_identifier(
        &self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<LocalIdentifier<'arena>, ParseError> {
        let token = stream.consume()?;

        if !token.kind.is_identifier_maybe_reserved() {
            return Err(stream.unexpected(Some(token), &[T![Identifier]]));
        }

        Ok(LocalIdentifier { span: token.span, value: token.value })
    }

    pub(crate) fn parse_qualified_identifier(
        &self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<QualifiedIdentifier<'arena>, ParseError> {
        let token = stream.eat(T![QualifiedIdentifier])?;

        Ok(QualifiedIdentifier { span: token.span, value: token.value })
    }

    pub(crate) fn parse_fully_qualified_identifier(
        &self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<FullyQualifiedIdentifier<'arena>, ParseError> {
        let token = stream.eat(T![FullyQualifiedIdentifier])?;

        Ok(FullyQualifiedIdentifier { span: token.span, value: token.value })
    }
}
