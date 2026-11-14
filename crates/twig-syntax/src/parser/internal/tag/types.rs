use crate::cst::Statement;
use crate::cst::StringLiteral;
use crate::cst::TokenSeparatedSequence;
use crate::cst::TypeDeclaration;
use crate::cst::Types;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigToken;
use crate::token::TwigTokenKind;
use mago_allocator::prelude::*;
use mago_span::Position;
use mago_span::Span;

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_types(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError<'arena>> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);
        let left_brace = self.stream.try_consume(TwigTokenKind::LeftBrace)?.map(|token| self.stream.span_of(&token));
        let mut declarations = self.new_vec();
        let mut commas = self.new_vec();
        while self.stream.peek_kind(0)?.is_some_and(|kind| !kind.is_close_block() && kind != TwigTokenKind::RightBrace)
        {
            if !declarations.is_empty() {
                commas.push(self.stream.expect_kind(TwigTokenKind::Comma, b"expected `,`")?);
                if self
                    .stream
                    .peek_kind(0)?
                    .is_some_and(|kind| kind.is_close_block() || kind == TwigTokenKind::RightBrace)
                {
                    break;
                }
            }

            let name = self.expect_flexible_identifier(b"expected variable name")?;
            let (question_mark, colon) = if let Some(token) = self.stream.try_consume(TwigTokenKind::QuestionColon)? {
                let span = self.stream.span_of(&token);
                if token.value != b"?:" {
                    return Err(ParseError::UnexpectedToken(b"expected `?:` without whitespace", span));
                }

                let middle = Position::new(span.start.offset + 1);
                (Some(Span::new(span.file_id, span.start, middle)), Span::new(span.file_id, middle, span.end))
            } else {
                let question_mark =
                    self.stream.try_consume(TwigTokenKind::Question)?.map(|token| self.stream.span_of(&token));
                let colon = self.stream.expect_kind(TwigTokenKind::Colon, b"expected `:`")?;
                (question_mark, self.stream.span_of(&colon))
            };
            let token = self
                .stream
                .lookahead(0)?
                .ok_or_else(|| self.stream.unexpected(None, &[TwigTokenKind::StringSingleQuoted]))?;
            if !matches!(token.kind, TwigTokenKind::StringSingleQuoted | TwigTokenKind::StringDoubleQuoted) {
                return Err(self.stream.unexpected(Some(token), &[TwigTokenKind::StringSingleQuoted]));
            }

            self.stream.consume()?;
            declarations.push(TypeDeclaration {
                name,
                question_mark,
                colon,
                value: StringLiteral { raw: token.value, span: self.stream.span_of(&token) },
            });
        }

        let right_brace = if left_brace.is_some() {
            Some(self.stream.expect_kind(TwigTokenKind::RightBrace, b"expected `}`")?)
                .map(|token| self.stream.span_of(&token))
        } else {
            None
        };
        let close_tag = self.stream.expect_block_end()?;
        Ok(Statement::Types(Types {
            open_tag,
            keyword,
            left_brace,
            declarations: TokenSeparatedSequence::new(declarations, commas),
            right_brace,
            close_tag,
        }))
    }
}
