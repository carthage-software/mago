use crate::T;
use crate::ast::ast::Modifier;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_modifier_sequence(&mut self) -> Result<Sequence<'arena, Modifier<'arena>>, ParseError> {
        let mut modifiers = self.new_vec();
        while let Some(modifier) = self.parse_optional_modifier()? {
            modifiers.push(modifier);
        }

        Ok(Sequence::new(modifiers))
    }

    pub(crate) fn parse_optional_read_visibility_modifier(&mut self) -> Result<Option<Modifier<'arena>>, ParseError> {
        Ok(Some(match self.stream.peek_kind(0)? {
            Some(T!["public"]) => Modifier::Public(self.expect_any_keyword()?),
            Some(T!["protected"]) => Modifier::Protected(self.expect_any_keyword()?),
            Some(T!["private"]) => Modifier::Private(self.expect_any_keyword()?),
            _ => return Ok(None),
        }))
    }

    pub(crate) fn parse_optional_modifier(&mut self) -> Result<Option<Modifier<'arena>>, ParseError> {
        Ok(Some(match self.stream.peek_kind(0)? {
            Some(T!["public"]) => Modifier::Public(self.expect_any_keyword()?),
            Some(T!["protected"]) => Modifier::Protected(self.expect_any_keyword()?),
            Some(T!["private"]) => Modifier::Private(self.expect_any_keyword()?),
            Some(T!["static"]) => Modifier::Static(self.expect_any_keyword()?),
            Some(T!["final"]) => Modifier::Final(self.expect_any_keyword()?),
            Some(T!["abstract"]) => Modifier::Abstract(self.expect_any_keyword()?),
            Some(T!["readonly"]) => Modifier::Readonly(self.expect_any_keyword()?),
            Some(T!["private(set)"]) => Modifier::PrivateSet(self.expect_any_keyword()?),
            Some(T!["protected(set)"]) => Modifier::ProtectedSet(self.expect_any_keyword()?),
            Some(T!["public(set)"]) => Modifier::PublicSet(self.expect_any_keyword()?),
            _ => return Ok(None),
        }))
    }
}
