use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax_core::cst::Sequence;

use crate::cst::identifier::Identifier;
use crate::cst::tag::MethodParameterList;
use crate::cst::tag::MethodTagValue;
use crate::cst::tag::MethodTagValueParameter;
use crate::cst::tag::MethodTagValueParameterDefault;
use crate::cst::tag::MethodTemplateParameter;
use crate::cst::tag::MethodTemplateParameterList;
use crate::cst::tag::TagValue;
use crate::cst::tag::TemplateTagValueVariance;
use crate::cst::tag::Visibility;
use crate::cst::r#type::ReferenceKind;
use crate::cst::r#type::Type;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

#[inline]
fn is_static_identifier(r#type: &Type<'_>) -> bool {
    matches!(r#type, Type::Reference(reference)
        if reference.parameters.is_none() && matches!(reference.kind, ReferenceKind::Static(_)))
}

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    fn identifier_from_type(&self, r#type: &Type<'arena>) -> Identifier<'arena> {
        if let Type::Reference(reference) = r#type
            && reference.parameters.is_none()
        {
            return match reference.kind {
                ReferenceKind::Identifier(identifier) => identifier,
                ReferenceKind::Self_(keyword) | ReferenceKind::Static(keyword) | ReferenceKind::Parent(keyword) => {
                    Identifier { span: keyword.span, value: keyword.value }
                }
            };
        }

        let span = r#type.span();

        Identifier { span, value: self.stream.raw_between(span.start, span.end) }
    }

    pub(crate) fn parse_method_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let visibility = self.parse_optional_method_visibility()?;

        let first = self.parse_type_without_conditional()?;
        let static_candidate = is_static_identifier(&first)
            && !self.stream.is_at(TokenKind::LeftParenthesis)
            && !self.stream.is_at(TokenKind::LeftAngleBracket);

        let (r#static, return_type, name) = if static_candidate {
            let candidate = self.parse_type_without_conditional()?;
            if self.stream.is_at(TokenKind::Identifier) {
                let return_type = self.alloc(candidate);
                let name = self.parse_identifier()?;

                (Some(first.span()), Some(return_type), name)
            } else {
                let name = self.identifier_from_type(&candidate);

                (None, Some(self.alloc(first)), name)
            }
        } else if self.stream.is_at(TokenKind::Identifier) {
            let return_type = self.alloc(first);
            let name = self.parse_identifier()?;

            (None, Some(return_type), name)
        } else {
            (None, None, self.identifier_from_type(&first))
        };

        let templates = self.parse_method_templates()?;
        let parameters = self.parse_method_parameters()?;
        let description = self.parse_optional_description(false)?;

        Ok(TagValue::Method(MethodTagValue {
            r#static,
            visibility,
            return_type,
            name,
            templates,
            parameters,
            description,
        }))
    }

    fn parse_optional_method_visibility(&mut self) -> Result<Option<Visibility<'arena>>, ParseError> {
        let Some(token) = self.stream.lookahead(0) else {
            return Ok(None);
        };

        if token.kind != TokenKind::Identifier
            || !(token.value.eq_ignore_ascii_case(b"public")
                || token.value.eq_ignore_ascii_case(b"protected")
                || token.value.eq_ignore_ascii_case(b"private"))
        {
            return Ok(None);
        }

        let keyword = self.parse_keyword()?;
        let visibility = if keyword.value.eq_ignore_ascii_case(b"public") {
            Visibility::Public(keyword)
        } else if keyword.value.eq_ignore_ascii_case(b"protected") {
            Visibility::Protected(keyword)
        } else {
            Visibility::Private(keyword)
        };

        Ok(Some(visibility))
    }

    fn parse_method_templates(&mut self) -> Result<Option<&'arena MethodTemplateParameterList<'arena>>, ParseError> {
        if !self.stream.is_at(TokenKind::LeftAngleBracket) {
            return Ok(None);
        }

        let less_than = self.stream.consume_span()?;

        let mut entries = self.new_vec::<MethodTemplateParameter<'arena>>();
        loop {
            let template = self.parse_template_tag_value(TemplateTagValueVariance::Invariant, false)?;
            let comma = if self.stream.is_at(TokenKind::Comma) { Some(self.stream.consume_span()?) } else { None };
            let has_comma = comma.is_some();
            entries.push(MethodTemplateParameter { template, comma });

            if !has_comma || self.stream.is_at(TokenKind::RightAngleBracket) {
                break;
            }
        }

        let greater_than = self.stream.eat_span(TokenKind::RightAngleBracket)?;

        Ok(Some(self.alloc(MethodTemplateParameterList { less_than, entries: Sequence::new(entries), greater_than })))
    }

    fn parse_method_parameters(&mut self) -> Result<&'arena MethodParameterList<'arena>, ParseError> {
        let left_parenthesis = self.stream.eat_span(TokenKind::LeftParenthesis)?;

        let mut entries = self.new_vec::<MethodTagValueParameter<'arena>>();
        if !self.stream.is_at(TokenKind::RightParenthesis) {
            loop {
                let parameter = self.parse_method_parameter()?;
                let has_comma = parameter.comma.is_some();
                entries.push(parameter);

                if !has_comma || self.stream.is_at(TokenKind::RightParenthesis) {
                    break;
                }
            }
        }

        let right_parenthesis = self.stream.eat_span(TokenKind::RightParenthesis)?;

        Ok(self.alloc(MethodParameterList { left_parenthesis, entries: Sequence::new(entries), right_parenthesis }))
    }

    fn parse_method_parameter(&mut self) -> Result<MethodTagValueParameter<'arena>, ParseError> {
        let r#type = if matches!(
            self.stream.peek_kind(0),
            Some(TokenKind::Ampersand | TokenKind::Ellipsis | TokenKind::Variable | TokenKind::ThisVariable)
        ) {
            None
        } else {
            let r#type = self.parse_type()?;

            Some(self.alloc(r#type))
        };

        let ampersand = if self.stream.is_at(TokenKind::Ampersand) { Some(self.stream.consume_span()?) } else { None };
        let ellipsis = if self.stream.is_at(TokenKind::Ellipsis) { Some(self.stream.consume_span()?) } else { None };
        let parameter = self.parse_variable()?;

        let default = if self.stream.is_at(TokenKind::Equals) {
            let equals = self.stream.consume_span()?;
            let value = self.parse_constant_expression()?;
            let value = self.alloc(value);

            Some(MethodTagValueParameterDefault { equals, value })
        } else {
            None
        };

        let comma = if self.stream.is_at(TokenKind::Comma) { Some(self.stream.consume_span()?) } else { None };

        Ok(MethodTagValueParameter { r#type, ampersand, ellipsis, parameter, default, comma })
    }
}
