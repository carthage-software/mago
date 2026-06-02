use mago_syntax_core::cst::Sequence;

use crate::cst::r#type::CallableType;
use crate::cst::r#type::CallableTypeKind;
use crate::cst::r#type::CallableTypeParameter;
use crate::cst::r#type::CallableTypeParameters;
use crate::cst::r#type::CallableTypeReturnType;
use crate::cst::r#type::CallableTypeSpecification;
use crate::cst::r#type::Type;
use crate::cst::variable::Variable;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::parser::internal::r#type::TypePrecedence;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_callable_type(&mut self, kind: CallableTypeKind) -> Result<Type<'arena>, ParseError> {
        let keyword = self.parse_keyword()?;
        let specification = self.parse_optional_callable_type_specifications()?;

        Ok(Type::Callable(CallableType { kind, keyword, specification }))
    }

    pub(crate) fn parse_callable_type_specifications(
        &mut self,
    ) -> Result<CallableTypeSpecification<'arena>, ParseError> {
        let left_parenthesis = self.stream.eat_span(TokenKind::LeftParenthesis)?;

        let mut entries = self.new_vec::<CallableTypeParameter<'arena>>();
        while !self.stream.is_at(TokenKind::RightParenthesis) {
            let parameter_type = if self.stream.is_at(TokenKind::Ellipsis) { None } else { Some(self.parse_type()?) };
            let ampersand =
                if self.stream.is_at(TokenKind::Ampersand) { Some(self.stream.consume_span()?) } else { None };
            let equals = if self.stream.is_at(TokenKind::Equals) { Some(self.stream.consume_span()?) } else { None };
            let ellipsis =
                if self.stream.is_at(TokenKind::Ellipsis) { Some(self.stream.consume_span()?) } else { None };
            let variable = if self.stream.is_at(TokenKind::Variable) {
                Some(Variable::from_token(self.stream.consume()?, self.file_id()))
            } else {
                None
            };
            let comma = if self.stream.is_at(TokenKind::Comma) { Some(self.stream.consume_span()?) } else { None };
            let has_comma = comma.is_some();
            entries.push(CallableTypeParameter { parameter_type, ampersand, equals, ellipsis, variable, comma });

            if !has_comma {
                break;
            }
        }

        let right_parenthesis = self.stream.eat_span(TokenKind::RightParenthesis)?;
        let parameters =
            CallableTypeParameters { left_parenthesis, entries: Sequence::new(entries), right_parenthesis };

        let return_type = if self.stream.is_at(TokenKind::Colon) {
            let colon = self.stream.consume_span()?;
            let return_type = self.parse_type_with_precedence(TypePrecedence::Callable)?;

            Some(CallableTypeReturnType { colon, return_type: self.alloc(return_type) })
        } else {
            None
        };

        Ok(CallableTypeSpecification { parameters, return_type })
    }

    pub(crate) fn parse_optional_callable_type_specifications(
        &mut self,
    ) -> Result<Option<CallableTypeSpecification<'arena>>, ParseError> {
        if self.stream.is_at(TokenKind::LeftParenthesis) {
            Ok(Some(self.parse_callable_type_specifications()?))
        } else {
            Ok(None)
        }
    }
}
