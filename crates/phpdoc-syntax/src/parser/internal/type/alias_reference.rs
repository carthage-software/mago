use crate::cst::identifier::Identifier;
use crate::cst::r#type::AliasName;
use crate::cst::r#type::AliasReferenceType;
use crate::cst::r#type::ReferenceKind;
use crate::cst::r#type::Type;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::parser::internal::r#type::keyword::TypeKeyword;
use crate::parser::internal::r#type::keyword::lookup_keyword;
use crate::token::TokenKind;
use mago_allocator::Arena;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_alias_reference_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let file_id = self.file_id();
        let exclamation = self.stream.consume_span()?;
        let class = {
            let next = self.stream.peek()?;
            match (next.kind == TokenKind::Identifier).then(|| lookup_keyword(next.value)).flatten() {
                Some(TypeKeyword::Self_) => ReferenceKind::Self_(self.parse_keyword()?),
                Some(TypeKeyword::Static) => ReferenceKind::Static(self.parse_keyword()?),
                Some(TypeKeyword::Parent) => ReferenceKind::Parent(self.parse_keyword()?),
                _ => ReferenceKind::Identifier(self.parse_identifier()?),
            }
        };
        let double_colon = self.stream.eat_span(TokenKind::ColonColon)?;

        let next = self.stream.peek()?;
        if next.kind != TokenKind::Identifier {
            return Err(ParseError::UnexpectedToken(next.span_for(file_id)));
        }

        let alias = if lookup_keyword(next.value).is_some() && !next.value.contains(&b'-') {
            AliasName::Keyword(self.parse_keyword()?)
        } else {
            AliasName::Identifier(Identifier::from_token(self.stream.consume()?, file_id))
        };

        Ok(Type::AliasReference(AliasReferenceType { exclamation, class, double_colon, alias }))
    }
}
