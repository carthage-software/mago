use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_ast::sequence::TokenSeparatedSequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::identifier::parse_local_identifier;
use crate::internal::statement::parse_statement;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_declare<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Declare<'i>, ParseError> {
    Ok(Declare {
        declare: utils::expect_keyword(stream, T!["declare"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        items: {
            let mut items = stream.vec();
            let mut commas = stream.vec();
            loop {
                let next = utils::peek(stream)?;
                if matches!(next.kind, T![")"]) {
                    break;
                }

                items.push(parse_declare_item(stream)?);

                match utils::maybe_expect(stream, T![","])? {
                    Some(comma) => {
                        commas.push(comma);
                    }
                    None => break,
                }
            }

            TokenSeparatedSequence::new(items, commas)
        },
        right_parenthesis: utils::expect_span(stream, T![")"])?,
        body: parse_declare_body(stream)?,
    })
}

#[inline]
pub fn parse_declare_item<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<DeclareItem<'i>, ParseError> {
    let name = parse_local_identifier(stream)?;
    let equal = utils::expect_span(stream, T!["="])?;
    let value = parse_expression(stream)?;

    Ok(DeclareItem { name, equal, value: stream.boxed(value) })
}

#[inline]
pub fn parse_declare_body<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<DeclareBody<'i>, ParseError> {
    let next = utils::peek(stream)?;

    Ok(match next.kind {
        T![":"] => DeclareBody::ColonDelimited(parse_declare_colon_delimited_body(stream)?),
        _ => {
            let statement = parse_statement(stream)?;

            DeclareBody::Statement(stream.boxed(statement))
        }
    })
}

#[inline]
pub fn parse_declare_colon_delimited_body<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<DeclareColonDelimitedBody<'i>, ParseError> {
    Ok(DeclareColonDelimitedBody {
        colon: utils::expect_span(stream, T![":"])?,
        statements: {
            let mut statements = stream.vec();
            loop {
                let next = utils::peek(stream)?;
                if matches!(next.kind, T!["enddeclare"]) {
                    break;
                }

                statements.push(parse_statement(stream)?);
            }
            Sequence::new(statements)
        },
        end_declare: utils::expect_keyword(stream, T!["enddeclare"])?,
        terminator: parse_terminator(stream)?,
    })
}
