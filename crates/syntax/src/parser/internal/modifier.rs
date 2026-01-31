use crate::T;
use crate::ast::ast::Modifier;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_modifier_sequence(
        &self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Sequence<'arena, Modifier<'arena>>, ParseError> {
        let mut modifiers = self.new_vec();
        while let Some(modifier) = self.parse_optional_modifier(stream)? {
            modifiers.push(modifier);
        }

        Ok(Sequence::new(modifiers))
    }

    pub(crate) fn parse_optional_read_visibility_modifier(
        &self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<Modifier<'arena>>, ParseError> {
        Ok(Some(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["public"]) => Modifier::Public(self.expect_any_keyword(stream)?),
            Some(T!["protected"]) => Modifier::Protected(self.expect_any_keyword(stream)?),
            Some(T!["private"]) => Modifier::Private(self.expect_any_keyword(stream)?),
            _ => return Ok(None),
        }))
    }

    pub(crate) fn parse_optional_modifier(
        &self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<Modifier<'arena>>, ParseError> {
        Ok(Some(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["public"]) => Modifier::Public(self.expect_any_keyword(stream)?),
            Some(T!["protected"]) => Modifier::Protected(self.expect_any_keyword(stream)?),
            Some(T!["private"]) => Modifier::Private(self.expect_any_keyword(stream)?),
            Some(T!["static"]) => Modifier::Static(self.expect_any_keyword(stream)?),
            Some(T!["final"]) => Modifier::Final(self.expect_any_keyword(stream)?),
            Some(T!["abstract"]) => Modifier::Abstract(self.expect_any_keyword(stream)?),
            Some(T!["readonly"]) => Modifier::Readonly(self.expect_any_keyword(stream)?),
            Some(T!["private(set)"]) => Modifier::PrivateSet(self.expect_any_keyword(stream)?),
            Some(T!["protected(set)"]) => Modifier::ProtectedSet(self.expect_any_keyword(stream)?),
            Some(T!["public(set)"]) => Modifier::PublicSet(self.expect_any_keyword(stream)?),
            _ => return Ok(None),
        }))
    }
}
