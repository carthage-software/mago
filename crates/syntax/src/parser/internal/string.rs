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
use crate::ast::ast::LiteralStringPart;
use crate::ast::ast::ShellExecuteString;
use crate::ast::ast::StringPart;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;
use crate::token::DocumentKind;
use crate::token::TokenKind;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_string(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<CompositeString<'arena>, ParseError> {
        let token = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;

        Ok(match token.kind {
            T!["\""] => CompositeString::Interpolated(self.parse_interpolated_string(stream)?),
            T!["`"] => CompositeString::ShellExecute(self.parse_shell_execute_string(stream)?),
            T!["<<<"] => CompositeString::Document(self.parse_document_string(stream)?),
            _ => {
                return Err(stream.unexpected(
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

    pub(crate) fn parse_interpolated_string(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<InterpolatedString<'arena>, ParseError> {
        let left_double_quote = stream.eat(T!["\""])?.span;
        let mut parts = self.new_vec();
        while let Some(part) = self.parse_optional_string_part(stream, T!["\""])? {
            parts.push(part);
        }

        let right_double_quote = stream.eat(T!["\""])?.span;

        Ok(InterpolatedString { left_double_quote, parts: Sequence::new(parts), right_double_quote })
    }

    pub(crate) fn parse_shell_execute_string(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<ShellExecuteString<'arena>, ParseError> {
        let left_backtick = stream.eat(T!["`"])?.span;
        let mut parts = self.new_vec();
        while let Some(part) = self.parse_optional_string_part(stream, T!["`"])? {
            parts.push(part);
        }

        let right_backtick = stream.eat(T!["`"])?.span;

        Ok(ShellExecuteString { left_backtick, parts: Sequence::new(parts), right_backtick })
    }

    pub(crate) fn parse_document_string(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<DocumentString<'arena>, ParseError> {
        let current = stream.consume()?;
        let (open, kind) = match current.kind {
            TokenKind::DocumentStart(DocumentKind::Heredoc) => (current.span, AstDocumentKind::Heredoc),
            TokenKind::DocumentStart(DocumentKind::Nowdoc) => (current.span, AstDocumentKind::Nowdoc),
            _ => {
                return Err(stream.unexpected(
                    Some(current),
                    &[TokenKind::DocumentStart(DocumentKind::Heredoc), TokenKind::DocumentStart(DocumentKind::Nowdoc)],
                ));
            }
        };

        let mut parts = self.new_vec();
        while let Some(part) = self.parse_optional_string_part(stream, T![DocumentEnd])? {
            parts.push(part);
        }

        let close = stream.eat(T![DocumentEnd])?;

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
            open,
            kind,
            indentation,
            parts: Sequence::new(parts),
            label: self.str(&label),
            close: close.span,
        })
    }

    pub(crate) fn parse_optional_string_part(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        closing_kind: TokenKind,
    ) -> Result<Option<StringPart<'arena>>, ParseError> {
        let token = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;
        Ok(match token.kind {
            T!["{"] => Some(StringPart::BracedExpression(self.parse_braced_expression_string_part(stream)?)),
            T![StringPart] => {
                let token = stream.consume()?;
                Some(StringPart::Literal(LiteralStringPart { span: token.span, value: token.value }))
            }
            kind if kind == closing_kind => None,
            _ => {
                let expr = self.parse_string_part_expression(stream)?;
                Some(StringPart::Expression(self.arena.alloc(expr)))
            }
        })
    }

    pub(crate) fn parse_braced_expression_string_part(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<BracedExpressionStringPart<'arena>, ParseError> {
        let left_brace = stream.eat(T!["{"])?.span;
        let expr = self.parse_expression(stream)?;
        let right_brace = stream.eat(T!["}"])?.span;

        Ok(BracedExpressionStringPart { left_brace, expression: self.arena.alloc(expr), right_brace })
    }

    fn parse_string_part_expression(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Expression<'arena>, ParseError> {
        let previous_state = self.state.within_string_interpolation;
        self.state.within_string_interpolation = true;
        let expression_result = self.parse_expression(stream);
        self.state.within_string_interpolation = previous_state;

        let expression = expression_result?;

        let Expression::ArrayAccess(ArrayAccess { array, left_bracket, index, right_bracket }) = expression else {
            return Ok(expression);
        };

        let index = index.clone();

        let Expression::ConstantAccess(ConstantAccess { name }) = index else {
            return Ok(Expression::ArrayAccess(ArrayAccess {
                array,
                left_bracket,
                index: self.arena.alloc(index),
                right_bracket,
            }));
        };

        Ok(Expression::ArrayAccess(ArrayAccess {
            array,
            left_bracket,
            index: self.arena.alloc(Expression::Identifier(name)),
            right_bracket,
        }))
    }
}
