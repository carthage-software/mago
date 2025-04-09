use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::statement::parse_statement;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_foreach(stream: &mut TokenStream<'_, '_>) -> Result<Foreach, ParseError> {
    Ok(Foreach {
        foreach: utils::expect_keyword(stream, T!["foreach"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        expression: Box::new(parse_expression(stream)?),
        r#as: utils::expect_keyword(stream, T!["as"])?,
        target: parse_foreach_target(stream)?,
        right_parenthesis: utils::expect_span(stream, T![")"])?,
        body: parse_foreach_body(stream)?,
    })
}

pub fn parse_foreach_target(stream: &mut TokenStream<'_, '_>) -> Result<ForeachTarget, ParseError> {
    let key_or_value = Box::new(parse_expression(stream)?);

    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["=>"]) => ForeachTarget::KeyValue(ForeachKeyValueTarget {
            key: key_or_value,
            double_arrow: utils::expect_any(stream)?.span,
            value: Box::new(parse_expression(stream)?),
        }),
        _ => ForeachTarget::Value(ForeachValueTarget { value: key_or_value }),
    })
}

pub fn parse_foreach_body(stream: &mut TokenStream<'_, '_>) -> Result<ForeachBody, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T![":"] => ForeachBody::ColonDelimited(parse_foreach_colon_delimited_body(stream)?),
        _ => ForeachBody::Statement(Box::new(parse_statement(stream)?)),
    })
}

pub fn parse_foreach_colon_delimited_body(
    stream: &mut TokenStream<'_, '_>,
) -> Result<ForeachColonDelimitedBody, ParseError> {
    Ok(ForeachColonDelimitedBody {
        colon: utils::expect_span(stream, T![":"])?,
        statements: {
            let mut statements = Vec::new();
            loop {
                if matches!(utils::peek(stream)?.kind, T!["endforeach"]) {
                    break;
                }

                statements.push(parse_statement(stream)?);
            }

            Sequence::new(statements)
        },
        end_foreach: utils::expect_keyword(stream, T!["endforeach"])?,
        terminator: parse_terminator(stream)?,
    })
}
