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

pub fn parse_declare(stream: &mut TokenStream<'_, '_>) -> Result<Declare, ParseError> {
    Ok(Declare {
        declare: utils::expect_keyword(stream, T!["declare"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        items: {
            let mut items = Vec::new();
            let mut commas = Vec::new();
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

pub fn parse_declare_item(stream: &mut TokenStream<'_, '_>) -> Result<DeclareItem, ParseError> {
    Ok(DeclareItem {
        name: parse_local_identifier(stream)?,
        equal: utils::expect_span(stream, T!["="])?,
        value: parse_expression(stream)?,
    })
}

pub fn parse_declare_body(stream: &mut TokenStream<'_, '_>) -> Result<DeclareBody, ParseError> {
    let next = utils::peek(stream)?;

    Ok(match next.kind {
        T![":"] => DeclareBody::ColonDelimited(parse_declare_colon_delimited_body(stream)?),
        _ => DeclareBody::Statement(Box::new(parse_statement(stream)?)),
    })
}

pub fn parse_declare_colon_delimited_body(
    stream: &mut TokenStream<'_, '_>,
) -> Result<DeclareColonDelimitedBody, ParseError> {
    Ok(DeclareColonDelimitedBody {
        colon: utils::expect_span(stream, T![":"])?,
        statements: {
            let mut statements = Vec::new();
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
