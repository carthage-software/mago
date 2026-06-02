use mago_syntax_core::cst::Sequence;

use crate::cst::r#type::GenericParameterEntry;
use crate::cst::r#type::GenericParameters;
use crate::cst::r#type::SingleGenericParameter;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_single_generic_parameter(&mut self) -> Result<SingleGenericParameter<'arena>, ParseError> {
        let less_than = self.stream.eat_span(TokenKind::LeftAngleBracket)?;
        let inner = self.parse_type()?;
        let comma = if self.stream.is_at(TokenKind::Comma) { Some(self.stream.consume_span()?) } else { None };
        let entry = self.alloc(GenericParameterEntry { inner, comma });
        let greater_than = self.stream.eat_span(TokenKind::RightAngleBracket)?;

        Ok(SingleGenericParameter { less_than, entry, greater_than })
    }

    pub(crate) fn parse_generic_parameters(&mut self) -> Result<GenericParameters<'arena>, ParseError> {
        let less_than = self.stream.eat_span(TokenKind::LeftAngleBracket)?;
        let mut entries = self.new_vec::<GenericParameterEntry<'arena>>();

        loop {
            let inner = self.parse_type()?;
            let comma = if self.stream.is_at(TokenKind::Comma) { Some(self.stream.consume_span()?) } else { None };
            let has_comma = comma.is_some();
            entries.push(GenericParameterEntry { inner, comma });

            if !has_comma || self.stream.is_at(TokenKind::RightAngleBracket) {
                break;
            }
        }

        let greater_than = self.stream.eat_span(TokenKind::RightAngleBracket)?;

        Ok(GenericParameters { less_than, entries: Sequence::new(entries), greater_than })
    }

    pub(crate) fn parse_single_generic_parameter_or_none(
        &mut self,
    ) -> Result<Option<SingleGenericParameter<'arena>>, ParseError> {
        if self.stream.is_at(TokenKind::LeftAngleBracket) {
            Ok(Some(self.parse_single_generic_parameter()?))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn parse_generic_parameters_or_none(&mut self) -> Result<Option<GenericParameters<'arena>>, ParseError> {
        if self.stream.is_at(TokenKind::LeftAngleBracket) {
            Ok(Some(self.parse_generic_parameters()?))
        } else {
            Ok(None)
        }
    }
}
