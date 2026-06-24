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
use mago_span::HasSpan;

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
                        && is_keyword(&token, TypeKeyword::Is)
                        && self.is_conditional_type_ahead() =>
                {
                    let subject = self.alloc(inner);

                    self.parse_conditional_type(subject)?
                }
                TokenKind::LeftBracket
                    if min_precedence <= TypePrecedence::Postfix && token.start.offset == inner.span().end.offset =>
                {
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

    /// Looks ahead to decide whether an `is` keyword opens a conditional type
    /// (`subject is target ? then : else`) or is merely free-text description that
    /// happens to begin with `is`, e.g. `@return bool is true when something`.
    ///
    /// It scans the tokens that would make up `[not] <target>` and only reports a
    /// conditional when that single target type is immediately followed by a `?`.
    fn is_conditional_type_ahead(&mut self) -> bool {
        let mut index = 1;

        if self.stream.lookahead(index).is_some_and(|token| is_keyword(&token, TypeKeyword::Not)) {
            index += 1;
        }

        let mut depth: usize = 0;
        let mut seen_atom = false;
        let mut after_operator = false;

        const SCAN_LIMIT: usize = 48;
        while index < SCAN_LIMIT {
            let Some(token) = self.stream.lookahead(index) else {
                return false;
            };

            if depth > 0 {
                match token.kind {
                    TokenKind::LeftParenthesis
                    | TokenKind::LeftAngleBracket
                    | TokenKind::LeftBracket
                    | TokenKind::LeftBrace => depth += 1,
                    TokenKind::RightParenthesis
                    | TokenKind::RightAngleBracket
                    | TokenKind::RightBracket
                    | TokenKind::RightBrace => {
                        depth -= 1;
                        if depth == 0 {
                            seen_atom = true;
                            after_operator = false;
                        }
                    }
                    _ => {}
                }

                index += 1;
                continue;
            }

            match token.kind {
                TokenKind::Question => {
                    if seen_atom && !after_operator {
                        return true;
                    }

                    seen_atom = false;
                    after_operator = true;
                }
                TokenKind::Pipe | TokenKind::Ampersand | TokenKind::ColonColon => {
                    if !seen_atom {
                        return false;
                    }

                    seen_atom = false;
                    after_operator = true;
                }
                TokenKind::LeftParenthesis
                | TokenKind::LeftAngleBracket
                | TokenKind::LeftBracket
                | TokenKind::LeftBrace => depth += 1,
                TokenKind::Identifier
                | TokenKind::Variable
                | TokenKind::ThisVariable
                | TokenKind::LiteralInteger
                | TokenKind::LiteralFloat
                | TokenKind::SingleQuotedString
                | TokenKind::DoubleQuotedString
                | TokenKind::Asterisk
                | TokenKind::Bang
                | TokenKind::Minus
                | TokenKind::Plus => {
                    if seen_atom && !after_operator {
                        return false;
                    }

                    seen_atom = true;
                    after_operator = false;
                }
                _ => return false,
            }

            index += 1;
        }

        false
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
