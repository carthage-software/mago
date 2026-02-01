use crate::T;
use crate::ast::ast::FullyQualifiedIdentifier;
use crate::ast::ast::Identifier;
use crate::ast::ast::LocalIdentifier;
use crate::ast::ast::QualifiedIdentifier;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_identifier(&mut self) -> Result<Identifier<'arena>, ParseError> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;

        Ok(match &token.kind {
            T![QualifiedIdentifier] => Identifier::Qualified(self.parse_qualified_identifier()?),
            T![FullyQualifiedIdentifier] => Identifier::FullyQualified(self.parse_fully_qualified_identifier()?),
            _ => Identifier::Local(self.parse_local_identifier()?),
        })
    }

    pub(crate) fn parse_local_identifier(&mut self) -> Result<LocalIdentifier<'arena>, ParseError> {
        let token = self.stream.consume()?;

        if !token.kind.is_identifier_maybe_reserved() {
            return Err(self.stream.unexpected(Some(token), &[T![Identifier]]));
        }

        Ok(LocalIdentifier { span: token.span, value: token.value })
    }

    pub(crate) fn parse_qualified_identifier(&mut self) -> Result<QualifiedIdentifier<'arena>, ParseError> {
        let token = self.stream.eat(T![QualifiedIdentifier])?;

        Ok(QualifiedIdentifier { span: token.span, value: token.value })
    }

    pub(crate) fn parse_fully_qualified_identifier(&mut self) -> Result<FullyQualifiedIdentifier<'arena>, ParseError> {
        let token = self.stream.eat(T![FullyQualifiedIdentifier])?;

        Ok(FullyQualifiedIdentifier { span: token.span, value: token.value })
    }
}
