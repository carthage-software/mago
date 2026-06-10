use crate::cst::r#type::ConditionalType;
use crate::cst::r#type::Type;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::parser::internal::r#type::TypePrecedence;
use crate::parser::internal::r#type::is_keyword;
use crate::parser::internal::r#type::keyword::TypeKeyword;
use crate::token::TokenKind;
use mago_allocator::Arena;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_conditional_type(&mut self, subject: &'arena Type<'arena>) -> Result<Type<'arena>, ParseError> {
        let is = self.parse_keyword()?;
        let not = if self.stream.lookahead(0).is_some_and(|t| is_keyword(&t, TypeKeyword::Not)) {
            Some(self.parse_keyword()?)
        } else {
            None
        };
        let target = self.parse_type_with_precedence(TypePrecedence::Conditional)?;
        let target = self.alloc(target);
        let question_mark = self.stream.eat_span(TokenKind::Question)?;
        let then = self.parse_type_with_precedence(TypePrecedence::Conditional)?;
        let then = self.alloc(then);
        let colon = self.stream.eat_span(TokenKind::Colon)?;
        let otherwise = self.parse_type_with_precedence(TypePrecedence::Conditional)?;
        let otherwise = self.alloc(otherwise);

        Ok(Type::Conditional(ConditionalType {
            subject,
            is,
            not,
            target,
            question_mark,
            then,
            colon,
            r#else: otherwise,
        }))
    }
}
