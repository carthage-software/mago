use crate::cst::tag::TagValue;
use crate::cst::tag::TypeAliasImportTagValue;
use crate::cst::tag::TypeAliasImportTagValueAs;
use crate::cst::tag::TypeAliasTagValue;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_type_alias_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let alias = self.parse_identifier()?;
        let equals = if self.stream.is_at(TokenKind::Equals) { Some(self.stream.consume_span()?) } else { None };
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);

        Ok(TagValue::TypeAlias(TypeAliasTagValue { alias, equals, r#type }))
    }

    pub(crate) fn parse_type_alias_import_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let imported_alias = self.parse_identifier()?;

        if !self.stream.is_at_value(b"from") {
            return Err(ParseError::UnexpectedToken(self.stream.peek()?.span_for(self.file_id())));
        }
        let from_keyword = self.parse_keyword()?;

        let imported_from = self.parse_identifier()?;

        let imported_as = if self.stream.is_at_value(b"as") {
            let keyword = self.parse_keyword()?;
            let local = self.parse_identifier()?;

            Some(TypeAliasImportTagValueAs { keyword, local })
        } else {
            None
        };

        Ok(TagValue::TypeAliasImport(TypeAliasImportTagValue {
            imported_alias,
            from_keyword,
            imported_from,
            imported_as,
        }))
    }
}
