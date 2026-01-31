use crate::T;
use crate::ast::ast::Namespace;
use crate::ast::ast::NamespaceBody;
use crate::ast::ast::NamespaceImplicitBody;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_namespace(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Namespace<'arena>, ParseError> {
        let namespace = self.expect_keyword(stream, T!["namespace"])?;
        let name = match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T![";" | "?>" | "{"]) => None,
            _ => Some(self.parse_identifier(stream)?),
        };
        let body = self.parse_namespace_body(stream)?;

        Ok(Namespace { namespace, name, body })
    }

    pub(crate) fn parse_namespace_body(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<NamespaceBody<'arena>, ParseError> {
        match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["{"]) => Ok(NamespaceBody::BraceDelimited(self.parse_block(stream)?)),
            _ => Ok(NamespaceBody::Implicit(self.parse_namespace_implicit_body(stream)?)),
        }
    }

    pub(crate) fn parse_namespace_implicit_body(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<NamespaceImplicitBody<'arena>, ParseError> {
        let terminator = self.parse_terminator(stream)?;
        let mut statements = self.new_vec();
        loop {
            let next = stream.lookahead(0)?.map(|t| t.kind);
            if matches!(next, None | Some(T!["namespace"])) {
                break;
            }

            match self.parse_statement(stream) {
                Ok(statement) => statements.push(statement),
                Err(err) => self.errors.push(err),
            }
        }

        Ok(NamespaceImplicitBody { terminator, statements: Sequence::new(statements) })
    }
}
