use crate::cst::identifier::Identifier;
use crate::cst::r#type::CallableType;
use crate::cst::r#type::CallableTypeKind;
use crate::cst::r#type::GlobalWildcardSelector;
use crate::cst::r#type::GlobalWildcardType;
use crate::cst::r#type::MemberReferenceSelector;
use crate::cst::r#type::MemberReferenceType;
use crate::cst::r#type::ReferenceType;
use crate::cst::r#type::Type;
use crate::cst::r#type::WildcardKind;
use crate::cst::r#type::WildcardType;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_reference_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let next = self.stream.peek()?;
        let file_id = self.file_id();

        if next.value == b"_" {
            let token = self.stream.consume()?;

            return Ok(Type::Wildcard(WildcardType { span: token.span_for(file_id), kind: WildcardKind::Underscore }));
        }

        if (next.value.eq_ignore_ascii_case(b"Closure") || next.value.eq_ignore_ascii_case(b"\\Closure"))
            && self.stream.lookahead(1).is_some_and(|t| t.kind == TokenKind::LeftParenthesis)
        {
            let keyword = self.parse_keyword()?;
            let specification = self.parse_callable_type_specifications()?;

            return Ok(Type::Callable(CallableType {
                kind: CallableTypeKind::Closure,
                keyword,
                specification: Some(specification),
            }));
        }

        let identifier = Identifier::from_token(self.stream.consume()?, file_id);

        if self.stream.is_at(TokenKind::ColonColon) {
            let double_colon = self.stream.consume_span()?;

            let member = if self.stream.is_at(TokenKind::Asterisk) {
                let asterisk = self.stream.consume_span()?;

                if self.is_at_member_identifier() {
                    MemberReferenceSelector::EndsWith(
                        asterisk,
                        Identifier::from_token(self.eat_member_identifier()?, file_id),
                    )
                } else {
                    MemberReferenceSelector::Wildcard(asterisk)
                }
            } else {
                let member_identifier = Identifier::from_token(self.eat_member_identifier()?, file_id);

                if self.stream.is_at(TokenKind::Asterisk) {
                    MemberReferenceSelector::StartsWith(member_identifier, self.stream.consume_span()?)
                } else {
                    MemberReferenceSelector::Identifier(member_identifier)
                }
            };

            Ok(Type::MemberReference(MemberReferenceType { class: identifier, double_colon, member }))
        } else if self.stream.is_at(TokenKind::Asterisk) {
            let asterisk = self.stream.consume_span()?;

            Ok(Type::GlobalWildcardReference(GlobalWildcardType {
                selector: GlobalWildcardSelector::StartsWith(identifier, asterisk),
            }))
        } else {
            let parameters = self.parse_generic_parameters_or_none()?;

            Ok(Type::Reference(ReferenceType { identifier, parameters }))
        }
    }
}
