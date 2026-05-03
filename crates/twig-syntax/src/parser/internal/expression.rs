use mago_span::HasSpan;

use crate::ast::Binary;
use crate::ast::BinaryOperator;
use crate::ast::Bool;
use crate::ast::Expression;
use crate::ast::Name;
use crate::ast::Null;
use crate::ast::Number;
use crate::ast::StringLiteral;
use crate::ast::Unary;
use crate::ast::UnaryOperator;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::is_keyword_usable_as_name;
use crate::parser::stream::looks_like_identifier;
use crate::token::TwigToken;
use crate::token::TwigTokenKind;

impl<'arena> Parser<'_, 'arena> {
    /// Parse an expression with the lowest (zero) minimum precedence.
    #[inline]
    pub(crate) fn parse_expression(&mut self) -> Result<Expression<'arena>, ParseError> {
        self.parse_expression_with_precedence(0)
    }

    /// Precedence-climbing expression parser.
    pub(crate) fn parse_expression_with_precedence(
        &mut self,
        min_precedence: u32,
    ) -> Result<Expression<'arena>, ParseError> {
        let mut left = self.parse_prefix()?;
        while let Some(token) = self.stream.lookahead(0)? {
            // Assignment (`=`): right-associative, lowest precedence.
            if min_precedence == 0 && token.kind == TwigTokenKind::Equal {
                let eq_tok = self.stream.consume()?;
                let operator = BinaryOperator::Assignment(self.stream.span_of(&eq_tok));
                let rhs = self.parse_expression()?;
                left = Expression::Binary(Binary { operator, lhs: self.alloc(left), rhs: self.alloc(rhs) });
                continue;
            }

            if let Some((operator, precedence, right_associative)) = classify_binary_operator(self, &token) {
                if precedence < min_precedence {
                    break;
                }

                self.stream.consume()?;
                let next_min = if right_associative { precedence } else { precedence + 1 };
                let rhs = self.parse_expression_with_precedence(next_min)?;
                left = Expression::Binary(Binary { operator, lhs: self.alloc(left), rhs: self.alloc(rhs) });
                continue;
            }

            if token.kind == TwigTokenKind::Question && min_precedence <= 3 {
                left = self.parse_conditional(left)?;
                continue;
            }

            if token.kind == TwigTokenKind::QuestionColon && min_precedence <= 3 {
                left = self.parse_elvis(left)?;
                continue;
            }

            if token.kind == TwigTokenKind::Pipe {
                let pipe_tok = self.stream.consume()?;
                let pipe = self.stream.span_of(&pipe_tok);
                left = self.parse_filter(left, pipe)?;
                continue;
            }

            if token.kind == TwigTokenKind::Dot {
                let dot_tok = self.stream.consume()?;
                let dot = self.stream.span_of(&dot_tok);
                left = self.parse_dot_access(left, dot, false)?;
                continue;
            }

            if token.kind == TwigTokenKind::QuestionDot {
                let dot_tok = self.stream.consume()?;
                let dot = self.stream.span_of(&dot_tok);
                left = self.parse_dot_access(left, dot, true)?;
                continue;
            }

            if token.kind == TwigTokenKind::LeftBracket {
                left = self.parse_bracket_access(left)?;
                continue;
            }

            if token.kind == TwigTokenKind::LeftParen && can_call(&left) {
                let argument_list = self.parse_argument_list()?;
                left = Expression::Call(crate::ast::Call { callee: self.alloc(left), argument_list });
                continue;
            }

            if token.kind == TwigTokenKind::Is {
                let is_tok = self.stream.consume()?;
                let is_keyword = self.keyword_from(&is_tok);
                left = self.parse_test(left, is_keyword)?;
                continue;
            }

            if token.kind == TwigTokenKind::FatArrow {
                let arrow_tok = self.stream.consume()?;
                let fat_arrow = self.stream.span_of(&arrow_tok);
                let parameters = self.arrow_parameters_from(&left);
                let body = self.parse_expression()?;
                left = Expression::ArrowFunction(crate::ast::ArrowFunction {
                    left_parenthesis: None,
                    parameters,
                    right_parenthesis: None,
                    fat_arrow,
                    body: self.alloc(body),
                });

                continue;
            }

            break;
        }
        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expression<'arena>, ParseError> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;

        let unary: Option<(UnaryOperator<'arena>, u32)> = match token.kind {
            TwigTokenKind::Minus => {
                self.stream.consume()?;
                Some((UnaryOperator::MinusSign(self.stream.span_of(&token)), 500))
            }
            TwigTokenKind::Plus => {
                self.stream.consume()?;
                Some((UnaryOperator::PlusSign(self.stream.span_of(&token)), 500))
            }
            TwigTokenKind::Not => {
                self.stream.consume()?;
                Some((UnaryOperator::Not(self.keyword_from(&token)), 50))
            }
            _ => None,
        };

        if let Some((operator, precedence)) = unary {
            let operand = self.parse_expression_with_precedence(precedence)?;
            return Ok(Expression::Unary(Unary { operator, operand: self.alloc(operand) }));
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression<'arena>, ParseError> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;

        match token.kind {
            TwigTokenKind::Number => {
                self.stream.consume()?;
                let is_float = token.value.contains('.') || token.value.contains('e') || token.value.contains('E');
                Ok(Expression::Number(Number { raw: token.value, is_float, span: self.stream.span_of(&token) }))
            }
            TwigTokenKind::StringSingleQuoted | TwigTokenKind::StringDoubleQuoted => {
                self.stream.consume()?;
                let first = Expression::String(StringLiteral { raw: token.value, span: self.stream.span_of(&token) });
                self.maybe_extend_string(first)
            }
            TwigTokenKind::DoubleQuoteStart => {
                let expression = self.parse_interpolated_string()?;
                self.maybe_extend_string(expression)
            }
            TwigTokenKind::Name => {
                self.stream.consume()?;
                let span = self.stream.span_of(&token);
                Ok(match token.value {
                    "true" | "TRUE" => Expression::Bool(Bool { value: true, span }),
                    "false" | "FALSE" => Expression::Bool(Bool { value: false, span }),
                    "null" | "NULL" | "none" | "NONE" => Expression::Null(Null { span }),
                    other => Expression::Name(Name { name: other, span }),
                })
            }
            TwigTokenKind::LeftBracket => self.parse_array(),
            TwigTokenKind::LeftParen => self.parse_group(),
            TwigTokenKind::LeftBrace => self.parse_hash_map(),
            kind if is_keyword_usable_as_name(kind) && looks_like_identifier(token.value) => {
                self.stream.consume()?;
                Ok(Expression::Name(Name { name: token.value, span: self.stream.span_of(&token) }))
            }
            _ => Err(ParseError::UnexpectedToken(
                format!("unexpected token {:?} of value {:?}", token.kind, token.value),
                self.stream.span_of(&token),
            )),
        }
    }

    /// Implicit string concatenation: `'a' 'b'` and `'a' "b"` become a
    /// `StringConcat` binary operation.
    fn maybe_extend_string(&mut self, first: Expression<'arena>) -> Result<Expression<'arena>, ParseError> {
        let mut left = first;
        while let Some(token) = self.stream.lookahead(0)? {
            match token.kind {
                TwigTokenKind::StringSingleQuoted | TwigTokenKind::StringDoubleQuoted => {
                    self.stream.consume()?;
                    let rhs = Expression::String(StringLiteral { raw: token.value, span: self.stream.span_of(&token) });
                    let concat_span = self.stream.span_of(&token);
                    left = Expression::Binary(Binary {
                        operator: BinaryOperator::StringConcat(concat_span),
                        lhs: self.alloc(left),
                        rhs: self.alloc(rhs),
                    });
                }
                TwigTokenKind::DoubleQuoteStart => {
                    let rhs = self.parse_interpolated_string()?;
                    let concat_span = rhs.span();
                    left = Expression::Binary(Binary {
                        operator: BinaryOperator::StringConcat(concat_span),
                        lhs: self.alloc(left),
                        rhs: self.alloc(rhs),
                    });
                }
                _ => break,
            }
        }
        Ok(left)
    }
}

fn can_call(expression: &Expression<'_>) -> bool {
    matches!(
        expression,
        Expression::Name(_)
            | Expression::GetAttribute(_)
            | Expression::GetItem(_)
            | Expression::Parenthesized(_)
            | Expression::Bool(_)
            | Expression::Null(_)
    )
}

fn classify_binary_operator<'arena>(
    parser: &Parser<'_, 'arena>,
    token: &TwigToken<'arena>,
) -> Option<(BinaryOperator<'arena>, u32, bool)> {
    let span = parser.stream.span_of(token);
    let keyword = || parser.keyword_from(token);
    match token.kind {
        TwigTokenKind::Or => Some((BinaryOperator::Or(keyword()), 10, false)),
        TwigTokenKind::Xor => Some((BinaryOperator::Xor(keyword()), 12, false)),
        TwigTokenKind::And => Some((BinaryOperator::And(keyword()), 15, false)),
        TwigTokenKind::BOr => Some((BinaryOperator::BitwiseOr(keyword()), 16, false)),
        TwigTokenKind::BXor => Some((BinaryOperator::BitwiseXor(keyword()), 17, false)),
        TwigTokenKind::BAnd => Some((BinaryOperator::BitwiseAnd(keyword()), 18, false)),
        TwigTokenKind::EqualEqual => Some((BinaryOperator::Equal(span), 20, false)),
        TwigTokenKind::BangEqual => Some((BinaryOperator::NotEqual(span), 20, false)),
        TwigTokenKind::Spaceship => Some((BinaryOperator::Spaceship(span), 20, false)),
        TwigTokenKind::LessThan => Some((BinaryOperator::LessThan(span), 20, false)),
        TwigTokenKind::GreaterThan => Some((BinaryOperator::GreaterThan(span), 20, false)),
        TwigTokenKind::LessThanEqual => Some((BinaryOperator::LessThanOrEqual(span), 20, false)),
        TwigTokenKind::GreaterThanEqual => Some((BinaryOperator::GreaterThanOrEqual(span), 20, false)),
        TwigTokenKind::EqualEqualEqual => Some((BinaryOperator::Identical(span), 20, false)),
        TwigTokenKind::BangEqualEqual => Some((BinaryOperator::NotIdentical(span), 20, false)),
        TwigTokenKind::In => Some((BinaryOperator::In(keyword()), 20, false)),
        TwigTokenKind::NotIn => Some((BinaryOperator::NotIn(keyword()), 20, false)),
        TwigTokenKind::Matches => Some((BinaryOperator::Matches(keyword()), 20, false)),
        TwigTokenKind::StartsWith => Some((BinaryOperator::StartsWith(keyword()), 20, false)),
        TwigTokenKind::EndsWith => Some((BinaryOperator::EndsWith(keyword()), 20, false)),
        TwigTokenKind::HasSome => Some((BinaryOperator::HasSome(keyword()), 20, false)),
        TwigTokenKind::HasEvery => Some((BinaryOperator::HasEvery(keyword()), 20, false)),
        TwigTokenKind::DotDot => Some((BinaryOperator::Range(span), 25, false)),
        TwigTokenKind::Plus => Some((BinaryOperator::Addition(span), 30, false)),
        TwigTokenKind::Minus => Some((BinaryOperator::Subtraction(span), 30, false)),
        TwigTokenKind::Tilde => Some((BinaryOperator::StringConcat(span), 40, false)),
        TwigTokenKind::Asterisk => Some((BinaryOperator::Multiplication(span), 60, false)),
        TwigTokenKind::Slash => Some((BinaryOperator::Division(span), 60, false)),
        TwigTokenKind::SlashSlash => Some((BinaryOperator::FloorDivision(span), 60, false)),
        TwigTokenKind::Percent => Some((BinaryOperator::Modulo(span), 60, false)),
        TwigTokenKind::AsteriskAsterisk => Some((BinaryOperator::Exponentiation(span), 200, true)),
        TwigTokenKind::QuestionQuestion => Some((BinaryOperator::NullCoalesce(span), 300, true)),
        _ => None,
    }
}
