use std::vec::Vec;

use mago_allocator::prelude::*;
use mago_database::file::HasFileId;
use mago_span::Span;
use mago_syntax_core::utils::parse_literal_string_in;

use crate::T;
use crate::cst::cst::ArrayAccess;
use crate::cst::cst::BracedExpressionStringPart;
use crate::cst::cst::CompositeString;
use crate::cst::cst::ConstantAccess;
use crate::cst::cst::DocumentIndentation;
use crate::cst::cst::DocumentKind as AstDocumentKind;
use crate::cst::cst::DocumentString;
use crate::cst::cst::Expression;
use crate::cst::cst::Identifier;
use crate::cst::cst::InterpolatedString;
use crate::cst::cst::Keyword;
use crate::cst::cst::Literal;
use crate::cst::cst::LiteralStringPart;
use crate::cst::cst::LocalIdentifier;
use crate::cst::cst::ShellExecuteString;
use crate::cst::cst::StringPart;
use crate::cst::cst::UnaryPrefix;
use crate::cst::cst::UnaryPrefixOperator;
use crate::cst::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::DocumentKind;
use crate::token::TokenKind;

/// How the literal bytes of a string part should be turned into their decoded value.
#[derive(Clone, Copy)]
pub(crate) enum LiteralPartDecoding {
    /// Apply the double-quoted escape rules (interpolated strings, shell-execute, heredoc).
    DoubleQuoted,
    /// Keep the bytes verbatim, with no escape decoding (nowdoc).
    Verbatim,
}

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_string(&mut self) -> Result<CompositeString<'arena>, ParseError> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;

        Ok(match token.kind {
            T!["\""] => CompositeString::Interpolated(self.parse_interpolated_string()?),
            T!["`"] => CompositeString::ShellExecute(self.parse_shell_execute_string()?),
            T!["<<<"] => CompositeString::Document(self.parse_document_string()?),
            _ => {
                return Err(self.stream.unexpected(
                    Some(token),
                    &[
                        T!["\""],
                        T!["`"],
                        TokenKind::DocumentStart(DocumentKind::Heredoc),
                        TokenKind::DocumentStart(DocumentKind::Nowdoc),
                    ],
                ));
            }
        })
    }

    pub(crate) fn parse_interpolated_string(&mut self) -> Result<InterpolatedString<'arena>, ParseError> {
        let token = self.stream.consume()?;
        let token_span = token.span_for(self.stream.file_id());
        let has_prefix = token.value.starts_with(b"b") || token.value.starts_with(b"B");
        let prefix = if has_prefix {
            let prefix_span = Span { start: token_span.start, end: token_span.start.forward(1), ..token_span };
            Some(Keyword { span: prefix_span, value: &token.value[..1] })
        } else {
            None
        };
        let left_double_quote =
            if has_prefix { Span { start: token_span.start.forward(1), ..token_span } } else { token_span };

        let mut parts = self.new_vec();
        while let Some(part) = self.parse_optional_string_part(T!["\""], LiteralPartDecoding::DoubleQuoted)? {
            parts.push(part);
        }

        let right_double_quote = self.stream.eat_span(T!["\""])?;

        Ok(InterpolatedString { prefix, left_double_quote, parts: Sequence::new(parts), right_double_quote })
    }

    pub(crate) fn parse_shell_execute_string(&mut self) -> Result<ShellExecuteString<'arena>, ParseError> {
        let left_backtick = self.stream.eat_span(T!["`"])?;
        let mut parts = self.new_vec();
        while let Some(part) = self.parse_optional_string_part(T!["`"], LiteralPartDecoding::DoubleQuoted)? {
            parts.push(part);
        }

        let right_backtick = self.stream.eat_span(T!["`"])?;

        Ok(ShellExecuteString { left_backtick, parts: Sequence::new(parts), right_backtick })
    }

    pub(crate) fn parse_document_string(&mut self) -> Result<DocumentString<'arena>, ParseError> {
        let current = self.stream.consume()?;
        let has_prefix = current.value.starts_with(b"b") || current.value.starts_with(b"B");
        let current_span = current.span_for(self.stream.file_id());
        let prefix = if has_prefix {
            let prefix_span =
                Span { start: current_span.start, end: current_span.start.forward(1), file_id: current_span.file_id };
            Some(Keyword { span: prefix_span, value: &current.value[..1] })
        } else {
            None
        };
        let open_span =
            if has_prefix { Span { start: current_span.start.forward(1), ..current_span } } else { current_span };
        let (open, kind, decoding) = match current.kind {
            TokenKind::DocumentStart(DocumentKind::Heredoc) => {
                (open_span, AstDocumentKind::Heredoc, LiteralPartDecoding::DoubleQuoted)
            }
            TokenKind::DocumentStart(DocumentKind::Nowdoc) => {
                (open_span, AstDocumentKind::Nowdoc, LiteralPartDecoding::Verbatim)
            }
            _ => {
                return Err(self.stream.unexpected(
                    Some(current),
                    &[TokenKind::DocumentStart(DocumentKind::Heredoc), TokenKind::DocumentStart(DocumentKind::Nowdoc)],
                ));
            }
        };

        let mut parts = self.new_vec();
        while let Some(part) = self.parse_optional_string_part(T![DocumentEnd], decoding)? {
            parts.push(part);
        }

        let close = self.stream.eat(T![DocumentEnd])?;

        let mut whitespaces = 0usize;
        let mut tabs = 0usize;
        let mut label: Vec<u8> = Vec::new();
        for &byte in close.value {
            match byte {
                b' ' => {
                    whitespaces += 1;
                }
                b'\t' => {
                    tabs += 1;
                }
                _ => {
                    label.push(byte);
                }
            }
        }

        let indentation = if tabs == 0 && whitespaces != 0 {
            DocumentIndentation::Whitespace(whitespaces)
        } else if tabs != 0 && whitespaces == 0 {
            DocumentIndentation::Tab(tabs)
        } else if tabs == 0 && whitespaces == 0 {
            DocumentIndentation::None
        } else {
            DocumentIndentation::Mixed(whitespaces, tabs)
        };

        Ok(DocumentString {
            prefix,
            open,
            kind,
            indentation,
            parts: Sequence::new(parts),
            label: self.bytes(&label),
            close: close.span_for(self.stream.file_id()),
        })
    }

    pub(crate) fn parse_optional_string_part(
        &mut self,
        closing_kind: TokenKind,
        decoding: LiteralPartDecoding,
    ) -> Result<Option<StringPart<'arena>>, ParseError> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
        Ok(match token.kind {
            T!["{"] => Some(StringPart::BracedExpression(self.parse_braced_expression_string_part()?)),
            T![StringPart] => {
                let token = self.stream.consume()?;
                let value = match decoding {
                    LiteralPartDecoding::DoubleQuoted => {
                        parse_literal_string_in(self.arena, token.value, Some(b'"'), false)
                    }
                    LiteralPartDecoding::Verbatim => Some(token.value),
                };

                Some(StringPart::Literal(LiteralStringPart {
                    span: token.span_for(self.stream.file_id()),
                    raw: token.value,
                    value,
                }))
            }
            kind if kind == closing_kind => None,
            _ => {
                let expr = self.parse_string_part_expression()?;
                Some(StringPart::Expression(expr))
            }
        })
    }

    pub(crate) fn parse_braced_expression_string_part(
        &mut self,
    ) -> Result<BracedExpressionStringPart<'arena>, ParseError> {
        let left_brace = self.stream.eat_span(T!["{"])?;
        let expr = self.parse_expression()?;
        let right_brace = self.stream.eat_span(T!["}"])?;

        Ok(BracedExpressionStringPart { left_brace, expression: expr, right_brace })
    }

    fn parse_string_part_expression(&mut self) -> Result<&'arena Expression<'arena>, ParseError> {
        let previous_state = self.state.within_string_interpolation;
        self.state.within_string_interpolation = true;
        let expression_result = self.parse_expression();
        self.state.within_string_interpolation = previous_state;

        let expression = expression_result?;

        let Expression::ArrayAccess(ArrayAccess { array, left_bracket, index, right_bracket }) = expression else {
            return Ok(expression);
        };

        let Some(index) = self.normalize_interpolated_offset(index) else {
            return Ok(expression);
        };

        Ok(self.arena.alloc(Expression::ArrayAccess(ArrayAccess {
            array,
            left_bracket: *left_bracket,
            index,
            right_bracket: *right_bracket,
        })))
    }

    /// Rewrites the offset of a simple-syntax interpolated array access (`"$a[offset]"`) to match
    /// PHP's special offset rules, returning `None` when the offset is already correct.
    ///
    /// In this position PHP treats the offset as a string key unless it is a canonical decimal
    /// integer. The only offsets that stay numeric are `"$a[1]"` and `"$a[-1]"`; everything else
    /// scalar becomes the string key of its raw lexeme:
    ///
    /// * A bareword (`"$a[key]"`) is the key `"key"`, so the `ConstantAccess` the expression
    ///   parser produced is turned back into a plain `Identifier`. `true`, `false`, and `null`
    ///   are barewords here too, so `"$a[true]"` is the key `"true"`.
    /// * A non-canonical numeric offset keeps its raw lexeme: `"$a[01]"`, `"$a[0x1]"`, and
    ///   `"$a[-0]"` are the keys `"01"`, `"0x1"`, and `"-0"`.
    /// * A float (`"$a[1.5]"`) is rejected by PHP, but Mago's interpolation lexer is lenient and
    ///   produces a float literal here; it is treated as the string key of its raw lexeme rather
    ///   than a (truncating) numeric key.
    fn normalize_interpolated_offset(&self, index: &'arena Expression<'arena>) -> Option<&'arena Expression<'arena>> {
        match index {
            Expression::ConstantAccess(ConstantAccess { name }) => {
                Some(self.arena.alloc(Expression::Identifier(*name)))
            }
            Expression::Literal(literal) => {
                let (raw, span) = match literal {
                    Literal::Integer(integer) if integer_offset_is_canonical(integer.raw) => return None,
                    Literal::Integer(integer) => (integer.raw, integer.span),
                    Literal::Float(float) => (float.raw, float.span),
                    Literal::True(keyword) | Literal::False(keyword) | Literal::Null(keyword) => {
                        (keyword.value, keyword.span)
                    }
                    Literal::String(_) => return None,
                };

                Some(self.string_key_offset(raw, span))
            }
            Expression::UnaryPrefix(UnaryPrefix {
                operator: UnaryPrefixOperator::Negation(minus),
                operand: Expression::Literal(Literal::Integer(integer)),
            }) => {
                let mut raw = self.new_vec();
                raw.push(b'-');
                raw.extend_from_slice(integer.raw);

                if integer_offset_is_canonical(&raw) {
                    return None;
                }

                Some(self.string_key_offset(self.bytes(&raw), minus.join(integer.span)))
            }
            _ => None,
        }
    }

    fn string_key_offset(&self, value: &'arena [u8], span: Span) -> &'arena Expression<'arena> {
        self.arena.alloc(Expression::Identifier(Identifier::Local(LocalIdentifier { span, value })))
    }
}

/// Returns `true` when `raw` is the canonical decimal spelling of an integer that fits in a
/// 64-bit PHP `int`, mirroring PHP's array-key normalization. Leading zeros (`"01"`), a negative
/// zero (`"-0"`), non-decimal bases (`"0x1"`), and out-of-range values are all non-canonical, so
/// PHP keeps them as string keys.
fn integer_offset_is_canonical(raw: &[u8]) -> bool {
    let digits = match raw {
        [b'-', rest @ ..] => rest,
        _ => raw,
    };

    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return false;
    }

    if digits[0] == b'0' {
        return raw == b"0";
    }

    std::str::from_utf8(raw).ok().and_then(|text| text.parse::<i64>().ok()).is_some()
}
