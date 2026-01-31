use crate::T;
use crate::ast::ast::Namespace;
use crate::ast::ast::NamespaceBody;
use crate::ast::ast::NamespaceImplicitBody;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_namespace(&mut self) -> Result<Namespace<'arena>, ParseError> {
        let namespace = self.expect_keyword(T!["namespace"])?;
        let name = match self.stream.lookahead(0)?.map(|t| t.kind) {
            Some(T![";" | "?>" | "{"]) => None,
            _ => Some(self.parse_identifier()?),
        };
        let body = self.parse_namespace_body()?;

        Ok(Namespace { namespace, name, body })
    }

    pub(crate) fn parse_namespace_body(&mut self) -> Result<NamespaceBody<'arena>, ParseError> {
        match self.stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["{"]) => Ok(NamespaceBody::BraceDelimited(self.parse_block()?)),
            _ => Ok(NamespaceBody::Implicit(self.parse_namespace_implicit_body()?)),
        }
    }

    pub(crate) fn parse_namespace_implicit_body(&mut self) -> Result<NamespaceImplicitBody<'arena>, ParseError> {
        let terminator = self.parse_terminator()?;
        let mut statements = self.new_vec();
        loop {
            let next = self.stream.lookahead(0)?.map(|t| t.kind);
            if matches!(next, None | Some(T!["namespace"])) {
                break;
            }

            match self.parse_statement() {
                Ok(statement) => statements.push(statement),
                Err(err) => self.errors.push(err),
            }
        }

        Ok(NamespaceImplicitBody { terminator, statements: Sequence::new(statements) })
    }
}
