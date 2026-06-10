use crate::cst::r#type::IntersectionType;
use crate::cst::r#type::NullableType;
use crate::cst::r#type::ParenthesizedType;
use crate::cst::r#type::TrailingPipeType;
use crate::cst::r#type::Type;
use crate::cst::r#type::UnionType;
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
    pub(crate) fn parse_type_with_precedence(
        &mut self,
        min_precedence: TypePrecedence,
    ) -> Result<Type<'arena>, ParseError> {
        self.stream.enter_recursion()?;
        let result = self.parse_type_with_precedence_inner(min_precedence);
        self.stream.leave_recursion();
        result
    }

    fn parse_type_with_precedence_inner(&mut self, min_precedence: TypePrecedence) -> Result<Type<'arena>, ParseError> {
        let mut inner = self.parse_primary_type()?;

        loop {
            let is_inner_nullable = matches!(inner, Type::Nullable(_));

            let Some(token) = self.stream.lookahead(0) else {
                return Ok(inner);
            };

            inner = match token.kind {
                TokenKind::Pipe if !is_inner_nullable && min_precedence <= TypePrecedence::Union => {
                    let pipe = self.stream.consume_span()?;

                    if self.is_at_union_closing_token() {
                        return Ok(Type::TrailingPipe(TrailingPipeType { inner: self.alloc(inner), pipe }));
                    }

                    let right = self.parse_type_with_precedence(TypePrecedence::Union)?;
                    if let Type::TrailingPipe(trailing) = right {
                        let union =
                            self.alloc(Type::Union(UnionType { left: self.alloc(inner), pipe, right: trailing.inner }));

                        return Ok(Type::TrailingPipe(TrailingPipeType { inner: union, pipe: trailing.pipe }));
                    }

                    let left = self.alloc(inner);
                    let right = self.alloc(right);

                    Type::Union(UnionType { left, pipe, right })
                }
                TokenKind::Ampersand
                    if !is_inner_nullable
                        && min_precedence <= TypePrecedence::Intersection
                        && !self
                            .stream
                            .lookahead(1)
                            .is_some_and(|t| matches!(t.kind, TokenKind::Variable | TokenKind::Ellipsis)) =>
                {
                    let left = self.alloc(inner);
                    let ampersand = self.stream.consume_span()?;
                    let right = self.parse_type_with_precedence(TypePrecedence::Intersection)?;
                    let right = self.alloc(right);

                    Type::Intersection(IntersectionType { left, ampersand, right })
                }
                TokenKind::Identifier
                    if !is_inner_nullable
                        && min_precedence <= TypePrecedence::Conditional
                        && is_keyword(&token, TypeKeyword::Is) =>
                {
                    let subject = self.alloc(inner);

                    self.parse_conditional_type(subject)?
                }
                TokenKind::LeftBracket if min_precedence <= TypePrecedence::Postfix => {
                    let left_bracket = self.stream.consume_span()?;

                    if self.stream.is_at(TokenKind::RightBracket) {
                        let inner_ref = self.alloc(inner);

                        self.parse_slice_type(inner_ref, left_bracket)?
                    } else {
                        let target = self.alloc(inner);

                        self.parse_index_access_type(target, left_bracket)?
                    }
                }
                _ => return Ok(inner),
            };
        }
    }

    #[inline]
    fn is_at_union_closing_token(&mut self) -> bool {
        match self.stream.peek_kind(0) {
            None => true,
            Some(kind) => matches!(
                kind,
                TokenKind::Comma
                    | TokenKind::RightParenthesis
                    | TokenKind::RightAngleBracket
                    | TokenKind::RightBrace
                    | TokenKind::RightBracket
                    | TokenKind::Colon
                    | TokenKind::Equals
                    | TokenKind::Variable
                    | TokenKind::Ellipsis
                    | TokenKind::Ampersand
            ),
        }
    }

    pub(crate) fn parse_nullable_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let question_mark = self.stream.consume_span()?;
        let inner = self.parse_type()?;

        Ok(Type::Nullable(NullableType { question_mark, inner: self.alloc(inner) }))
    }

    pub(crate) fn parse_parenthesized_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let left_parenthesis = self.stream.consume_span()?;
        let inner = self.parse_type()?;
        let inner = self.alloc(inner);
        let right_parenthesis = self.stream.eat_span(TokenKind::RightParenthesis)?;

        Ok(Type::Parenthesized(ParenthesizedType { left_parenthesis, inner, right_parenthesis }))
    }
}
