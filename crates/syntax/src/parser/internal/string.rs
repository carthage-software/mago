use crate::T;
use crate::ast::ast::DocumentKind as AstDocumentKind;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;
use crate::token::DocumentKind;
use crate::token::TokenKind;

pub fn parse_string<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<CompositeString<'arena>, ParseError> {
    let token = utils::peek(stream)?;

    Ok(match token.kind {
        T!["\""] => CompositeString::Interpolated(parse_interpolated_string(stream)?),
        T!["`"] => CompositeString::ShellExecute(parse_shell_execute_string(stream)?),
        T!["<<<"] => CompositeString::Document(parse_document_string(stream)?),
        _ => {
            return Err(utils::unexpected(
                stream,
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

pub fn parse_interpolated_string<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<InterpolatedString<'arena>, ParseError> {
    let left_double_quote = utils::expect_span(stream, T!["\""])?;
    let mut parts = stream.new_vec();
    while let Some(part) = parse_optional_string_part(stream, T!["\""])? {
        parts.push(part);
    }

    let right_double_quote = utils::expect_span(stream, T!["\""])?;

    Ok(InterpolatedString { left_double_quote, parts: Sequence::new(parts), right_double_quote })
}

pub fn parse_shell_execute_string<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<ShellExecuteString<'arena>, ParseError> {
    let left_backtick = utils::expect_span(stream, T!["`"])?;
    let mut parts = stream.new_vec();
    while let Some(part) = parse_optional_string_part(stream, T!["`"])? {
        parts.push(part);
    }

    let right_backtick = utils::expect_span(stream, T!["`"])?;

    Ok(ShellExecuteString { left_backtick, parts: Sequence::new(parts), right_backtick })
}

pub fn parse_document_string<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<DocumentString<'arena>, ParseError> {
    let current = utils::expect_any(stream)?;
    let (open, kind) = match current.kind {
        TokenKind::DocumentStart(DocumentKind::Heredoc) => (current.span, AstDocumentKind::Heredoc),
        TokenKind::DocumentStart(DocumentKind::Nowdoc) => (current.span, AstDocumentKind::Nowdoc),
        _ => {
            return Err(utils::unexpected(
                stream,
                Some(current),
                &[TokenKind::DocumentStart(DocumentKind::Heredoc), TokenKind::DocumentStart(DocumentKind::Nowdoc)],
            ));
        }
    };

    let mut parts = stream.new_vec();
    while let Some(part) = parse_optional_string_part(stream, T![DocumentEnd])? {
        parts.push(part);
    }

    let close = utils::expect(stream, T![DocumentEnd])?;

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
        label: stream.str(&label),
        close: close.span,
    })
}

pub fn parse_optional_string_part<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    closing_kind: TokenKind,
) -> Result<Option<StringPart<'arena>>, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T!["{"] => Some(StringPart::BracedExpression(parse_braced_expression_string_part(stream)?)),
        T![StringPart] => {
            let token = utils::expect_any(stream)?;

            Some(StringPart::Literal(LiteralStringPart { span: token.span, value: token.value }))
        }
        kind if kind == closing_kind => None,
        _ => {
            let expression = parse_string_part_expression(stream)?;

            Some(StringPart::Expression(stream.alloc(expression)))
        }
    })
}

pub fn parse_braced_expression_string_part<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<BracedExpressionStringPart<'arena>, ParseError> {
    let left_brace = utils::expect_span(stream, T!["{"])?;
    let expression = parse_expression(stream)?;
    let right_brace = utils::expect_span(stream, T!["}"])?;

    Ok(BracedExpressionStringPart { left_brace, expression: stream.alloc(expression), right_brace })
}

fn parse_string_part_expression<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Expression<'arena>, ParseError> {
    let expression = parse_expression(stream)?;

    let Expression::ArrayAccess(ArrayAccess { array, left_bracket, index, right_bracket }) = expression else {
        return Ok(expression);
    };

    let index = index.clone();

    let Expression::ConstantAccess(ConstantAccess { name }) = index else {
        return Ok(Expression::ArrayAccess(ArrayAccess {
            array,
            left_bracket,
            index: stream.alloc(index),
            right_bracket,
        }));
    };

    Ok(Expression::ArrayAccess(ArrayAccess {
        array,
        left_bracket,
        index: stream.alloc(Expression::Identifier(name)),
        right_bracket,
    }))
}
