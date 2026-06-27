use mago_allocator::Arena;
use mago_syntax_core::cst::Sequence;

use crate::cst::keyword::Keyword;
use crate::cst::r#type::GenericParameterEntry;
use crate::cst::r#type::GenericParameterVariance;
use crate::cst::r#type::GenericParameters;
use crate::cst::r#type::SingleGenericParameter;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    fn parse_optional_generic_parameter_variance(
        &mut self,
    ) -> Result<Option<GenericParameterVariance<'arena>>, ParseError> {
        let Some(token) = self.stream.lookahead(0) else {
            return Ok(None);
        };

        if token.kind != TokenKind::Identifier {
            return Ok(None);
        }

        if matches!(self.stream.peek_kind(1), None | Some(TokenKind::Comma | TokenKind::RightAngleBracket)) {
            return Ok(None);
        }

        let variance = if token.value.eq_ignore_ascii_case(b"covariant") {
            GenericParameterVariance::Covariant(Keyword::from_token(self.stream.consume()?, self.file_id()))
        } else if token.value.eq_ignore_ascii_case(b"contravariant") {
            GenericParameterVariance::Contravariant(Keyword::from_token(self.stream.consume()?, self.file_id()))
        } else {
            return Ok(None);
        };

        Ok(Some(variance))
    }

    pub(crate) fn parse_single_generic_parameter(&mut self) -> Result<SingleGenericParameter<'arena>, ParseError> {
        let less_than = self.stream.eat_span(TokenKind::LeftAngleBracket)?;
        let variance = self.parse_optional_generic_parameter_variance()?;
        let inner = self.parse_type()?;
        let comma = if self.stream.is_at(TokenKind::Comma) { Some(self.stream.consume_span()?) } else { None };
        let entry = self.alloc(GenericParameterEntry { variance, inner, comma });
        let greater_than = self.stream.eat_span(TokenKind::RightAngleBracket)?;

        Ok(SingleGenericParameter { less_than, entry, greater_than })
    }

    pub(crate) fn parse_generic_parameters(&mut self) -> Result<GenericParameters<'arena>, ParseError> {
        let less_than = self.stream.eat_span(TokenKind::LeftAngleBracket)?;
        let mut entries = self.new_vec::<GenericParameterEntry<'arena>>();

        loop {
            let variance = self.parse_optional_generic_parameter_variance()?;
            let inner = self.parse_type()?;
            let comma = if self.stream.is_at(TokenKind::Comma) { Some(self.stream.consume_span()?) } else { None };
            let has_comma = comma.is_some();
            entries.push(GenericParameterEntry { variance, inner, comma });

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
