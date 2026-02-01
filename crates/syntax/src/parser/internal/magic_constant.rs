use crate::T;
use crate::ast::ast::MagicConstant;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_magic_constant(&mut self) -> Result<MagicConstant<'arena>, ParseError> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;

        Ok(match token.kind {
            T!["__CLASS__"] => MagicConstant::Class(self.parse_local_identifier()?),
            T!["__DIR__"] => MagicConstant::Directory(self.parse_local_identifier()?),
            T!["__FILE__"] => MagicConstant::File(self.parse_local_identifier()?),
            T!["__FUNCTION__"] => MagicConstant::Function(self.parse_local_identifier()?),
            T!["__LINE__"] => MagicConstant::Line(self.parse_local_identifier()?),
            T!["__METHOD__"] => MagicConstant::Method(self.parse_local_identifier()?),
            T!["__NAMESPACE__"] => MagicConstant::Namespace(self.parse_local_identifier()?),
            T!["__PROPERTY__"] => MagicConstant::Property(self.parse_local_identifier()?),
            T!["__TRAIT__"] => MagicConstant::Trait(self.parse_local_identifier()?),
            _ => {
                return Err(self.stream.unexpected(
                    Some(token),
                    T![
                        "__CLASS__",
                        "__DIR__",
                        "__FILE__",
                        "__FUNCTION__",
                        "__LINE__",
                        "__METHOD__",
                        "__NAMESPACE__",
                        "__PROPERTY__",
                        "__TRAIT__"
                    ],
                ));
            }
        })
    }
}
