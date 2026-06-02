use mago_span::Span;
use mago_syntax_core::cst::TokenSeparatedSequence;

use crate::cst::identifier::Identifier;
use crate::cst::tag::ExtendsTagValue;
use crate::cst::tag::ImplementsTagValue;
use crate::cst::tag::InheritorsTagValue;
use crate::cst::tag::RequireExtendsTagValue;
use crate::cst::tag::RequireImplementsTagValue;
use crate::cst::tag::SealedTagValue;
use crate::cst::tag::TagValue;
use crate::cst::tag::UsesTagValue;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_extends_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(TagValue::Extends(ExtendsTagValue { r#type, description }))
    }

    pub(crate) fn parse_implements_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(TagValue::Implements(ImplementsTagValue { r#type, description }))
    }

    pub(crate) fn parse_uses_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(TagValue::Uses(UsesTagValue { r#type, description }))
    }

    pub(crate) fn parse_require_extends_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(TagValue::RequireExtends(RequireExtendsTagValue { r#type, description }))
    }

    pub(crate) fn parse_require_implements_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(TagValue::RequireImplements(RequireImplementsTagValue { r#type, description }))
    }

    pub(crate) fn parse_sealed_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(TagValue::Sealed(SealedTagValue { r#type, description }))
    }

    pub(crate) fn parse_inheritors_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let file_id = self.file_id();
        let start = self.stream.peek()?.start;

        let mut nodes = self.new_vec::<Identifier<'arena>>();
        let mut separators = self.new_vec();

        loop {
            nodes.push(self.parse_identifier()?);

            if self.stream.is_at(TokenKind::Pipe) {
                separators.push(self.stream.consume()?);
            } else {
                break;
            }

            if !self.stream.is_at(TokenKind::Identifier) {
                break;
            }
        }

        let span = Span::new(file_id, start, self.stream.current_position());

        Ok(TagValue::Inheritors(InheritorsTagValue {
            span,
            inheritors: TokenSeparatedSequence::new(nodes, separators),
        }))
    }
}
