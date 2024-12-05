use fennec_ast::ast::DocumentKind as AstDocumentKind;
use fennec_ast::ast::*;
use fennec_ast::sequence::Sequence;
use fennec_token::DocumentKind;
use fennec_token::TokenKind;
use fennec_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

pub fn parse_string<'a, 'i>(stream: &mut TokenStream<'a, 'i>) -> Result<CompositeString, ParseError> {
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
            ))
        }
    })
}

pub fn parse_interpolated_string<'a, 'i>(stream: &mut TokenStream<'a, 'i>) -> Result<InterpolatedString, ParseError> {
    let left_double_quote = utils::expect_span(stream, T!["\""])?;
    let mut parts = vec![];
    while let Some(part) = parse_optional_string_part(stream, T!["\""])? {
        parts.push(part);
    }

    let right_double_quote = utils::expect_span(stream, T!["\""])?;

    Ok(InterpolatedString { left_double_quote, parts: Sequence::new(parts), right_double_quote })
}

pub fn parse_shell_execute_string<'a, 'i>(stream: &mut TokenStream<'a, 'i>) -> Result<ShellExecuteString, ParseError> {
    let left_backtick = utils::expect_span(stream, T!["`"])?;
    let mut parts = vec![];
    while let Some(part) = parse_optional_string_part(stream, T!["`"])? {
        parts.push(part);
    }

    let right_backtick = utils::expect_span(stream, T!["`"])?;

    Ok(ShellExecuteString { left_backtick, parts: Sequence::new(parts), right_backtick })
}

pub fn parse_document_string<'a, 'i>(stream: &mut TokenStream<'a, 'i>) -> Result<DocumentString, ParseError> {
    let current = utils::expect_any(stream)?;
    let (open, kind) = match current.kind {
        TokenKind::DocumentStart(DocumentKind::Heredoc) => (current.span, AstDocumentKind::Heredoc),
        TokenKind::DocumentStart(DocumentKind::Nowdoc) => (current.span, AstDocumentKind::Nowdoc),
        _ => {
            return Err(utils::unexpected(
                stream,
                Some(current),
                &[TokenKind::DocumentStart(DocumentKind::Heredoc), TokenKind::DocumentStart(DocumentKind::Nowdoc)],
            ))
        }
    };

    let mut parts = vec![];
    while let Some(part) = parse_optional_string_part(stream, T![DocumentEnd])? {
        parts.push(part);
    }

    let close = utils::expect(stream, T![DocumentEnd])?;

    let value = stream.interner().lookup(&close.value);

    let mut whitespaces = 0usize;
    let mut tabs = 0usize;
    let mut label = std::string::String::new();
    for char in value.chars() {
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

    let label = stream.interner().intern(label);

    Ok(DocumentString { open, kind, indentation, parts: Sequence::new(parts), label, close: close.span })
}

pub fn parse_optional_string_part<'a, 'i>(
    stream: &mut TokenStream<'a, 'i>,
    closing_kind: TokenKind,
) -> Result<Option<StringPart>, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T!["{"] => Some(StringPart::BracedExpression(parse_braced_expression_string_part(stream)?)),
        T![StringPart] => {
            let token = utils::expect_any(stream)?;

            Some(StringPart::Literal(LiteralStringPart { span: token.span, value: token.value }))
        }
        kind if kind == closing_kind => None,
        _ => Some(StringPart::Expression(Box::new(parse_expression(stream)?))),
    })
}
pub fn parse_braced_expression_string_part<'a, 'i>(
    stream: &mut TokenStream<'a, 'i>,
) -> Result<BracedExpressionStringPart, ParseError> {
    let left_brace = utils::expect_span(stream, T!["{"])?;
    let expression = Box::new(parse_expression(stream)?);
    let right_brace = utils::expect_span(stream, T!["}"])?;

    Ok(BracedExpressionStringPart { left_brace, expression, right_brace })
}
