use mago_database::file::HasFileId;

use mago_span::Span;

use crate::T;
use crate::ast::ast::ArrayAccess;
use crate::ast::ast::BracedExpressionStringPart;
use crate::ast::ast::CompositeString;
use crate::ast::ast::ConstantAccess;
use crate::ast::ast::DocumentIndentation;
use crate::ast::ast::DocumentKind as AstDocumentKind;
use crate::ast::ast::DocumentString;
use crate::ast::ast::Expression;
use crate::ast::ast::InterpolatedString;
use crate::ast::ast::Keyword;
use crate::ast::ast::LiteralStringPart;
use crate::ast::ast::ShellExecuteString;
use crate::ast::ast::StringPart;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::DocumentKind;
use crate::token::TokenKind;

impl<'arena> Parser<'_, 'arena> {
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
        let has_prefix = token.value.starts_with('b') || token.value.starts_with('B');
        let prefix = if has_prefix {
            let prefix_span = Span { start: token_span.start, end: token_span.start.forward(1), ..token_span };
            Some(Keyword { span: prefix_span, value: &token.value[..1] })
        } else {
            None
        };
        let left_double_quote =
            if has_prefix { Span { start: token_span.start.forward(1), ..token_span } } else { token_span };

        let mut parts = self.new_vec();
        while let Some(part) = self.parse_optional_string_part(T!["\""])? {
            parts.push(part);
        }

        let right_double_quote = self.stream.eat_span(T!["\""])?;

        Ok(InterpolatedString { prefix, left_double_quote, parts: Sequence::new(parts), right_double_quote })
    }

    pub(crate) fn parse_shell_execute_string(&mut self) -> Result<ShellExecuteString<'arena>, ParseError> {
        let left_backtick = self.stream.eat_span(T!["`"])?;
        let mut parts = self.new_vec();
        while let Some(part) = self.parse_optional_string_part(T!["`"])? {
            parts.push(part);
        }

        let right_backtick = self.stream.eat_span(T!["`"])?;

        Ok(ShellExecuteString { left_backtick, parts: Sequence::new(parts), right_backtick })
    }

    pub(crate) fn parse_document_string(&mut self) -> Result<DocumentString<'arena>, ParseError> {
        let current = self.stream.consume()?;
        let has_prefix = current.value.starts_with('b') || current.value.starts_with('B');
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
        let (open, kind) = match current.kind {
            TokenKind::DocumentStart(DocumentKind::Heredoc) => (open_span, AstDocumentKind::Heredoc),
            TokenKind::DocumentStart(DocumentKind::Nowdoc) => (open_span, AstDocumentKind::Nowdoc),
            _ => {
                return Err(self.stream.unexpected(
                    Some(current),
                    &[TokenKind::DocumentStart(DocumentKind::Heredoc), TokenKind::DocumentStart(DocumentKind::Nowdoc)],
                ));
            }
        };

        let mut parts = self.new_vec();
        while let Some(part) = self.parse_optional_string_part(T![DocumentEnd])? {
            parts.push(part);
        }

        let close = self.stream.eat(T![DocumentEnd])?;

        let mut whitespaces = 0usize;
        let mut tabs = 0usize;
        let mut label = std::string::String::new();
        for char in close.value.chars() {
            match char {
                ' ' => {
                    whitespaces += 1;
                }
                '\t' => {
                    tabs += 1;
                }
                _ => {
                    label.push(char);
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
            label: self.str(&label),
            close: close.span_for(self.stream.file_id()),
        })
    }

    pub(crate) fn parse_optional_string_part(
        &mut self,
        closing_kind: TokenKind,
    ) -> Result<Option<StringPart<'arena>>, ParseError> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
        Ok(match token.kind {
            T!["{"] => Some(StringPart::BracedExpression(self.parse_braced_expression_string_part()?)),
            T![StringPart] => {
                let token = self.stream.consume()?;
                Some(StringPart::Literal(LiteralStringPart {
                    span: token.span_for(self.stream.file_id()),
                    value: token.value,
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

        let Expression::ConstantAccess(ConstantAccess { name }) = index else {
            return Ok(expression);
        };

        Ok(self.arena.alloc(Expression::ArrayAccess(ArrayAccess {
            array,
            left_bracket: *left_bracket,
            index: self.arena.alloc(Expression::Identifier(*name)),
            right_bracket: *right_bracket,
        })))
    }
}
