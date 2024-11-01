use ordered_float::OrderedFloat;

use fennec_ast::ast::*;
use fennec_span::Position;
use fennec_token::T;

use crate::error::ParseError;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

pub fn parse_literal<'a, 'i>(stream: &mut TokenStream<'a, 'i>) -> Result<Literal, ParseError> {
    let token = utils::expect_any(stream)?;

    Ok(match &token.kind {
        T![LiteralFloat] => Literal::Float(LiteralFloat {
            span: token.span,
            raw: token.value,
            value: OrderedFloat(parse_literal_float(stream.interner().lookup(token.value), &token.span.start)),
        }),
        T![LiteralInteger] => Literal::Integer(LiteralInteger {
            span: token.span,
            raw: token.value,
            value: parse_literal_integer(stream.interner().lookup(token.value), &token.span.start),
        }),
        T!["true"] => Literal::True(utils::to_keyword(token)),
        T!["false"] => Literal::False(utils::to_keyword(token)),
        T!["null"] => Literal::Null(utils::to_keyword(token)),
        T![LiteralString] => {
            let value = stream.interner().lookup(token.value);

            let kind =
                if value.starts_with('"') { LiteralStringKind::DoubleQuoted } else { LiteralStringKind::SingleQuoted };

            Literal::String(LiteralString { kind, span: token.span, value: token.value })
        }
        T![PartialLiteralString] => {
            let value = stream.interner().lookup(token.value);

            let kind =
                if value.starts_with('"') { LiteralStringKind::DoubleQuoted } else { LiteralStringKind::SingleQuoted };

            return Err(ParseError::UnclosedLiteralString(kind, token.span));
        }
        _ => {
            return Err(utils::unexpected(
                stream,
                Some(token),
                T!["true", "false", "null", LiteralFloat, LiteralInteger, LiteralString, PartialLiteralString],
            ))
        }
    })
}

fn parse_literal_float(value: &str, at: &Position) -> f64 {
    let source = value.replace("_", "");

    source.parse::<f64>().expect(&format!("failed to parse float `{}` at {}; this should never happen.", source, at))
}

fn parse_literal_integer(value: &str, at: &Position) -> Option<u64> {
    let source = value.replace("_", "");

    Some(match source.as_bytes() {
        [b'0', b'x' | b'X', ..] => u64::from_str_radix(&source.as_str()[2..], 16)
            .expect(&format!("failed to parse hex integer `{}` at `{}`; this should never happen.", source, at)),
        [b'0', b'o' | b'O', ..] => u64::from_str_radix(&source.as_str()[2..], 8)
            .expect(&format!("failed to parse octal integer `{}` at `{}`; this should never happen.", source, at)),
        _ => return source.parse::<u64>().ok(),
    })
}
