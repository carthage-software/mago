use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::identifier::parse_local_identifier;
use crate::parser::internal::statement::parse_statement;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_declare<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Declare<'arena>, ParseError> {
    Ok(Declare {
        declare: utils::expect_keyword(stream, T!["declare"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        items: {
            let mut items = stream.new_vec();
            let mut commas = stream.new_vec();
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

pub fn parse_declare_item<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<DeclareItem<'arena>, ParseError> {
    Ok(DeclareItem {
        name: parse_local_identifier(stream)?,
        equal: utils::expect_span(stream, T!["="])?,
        value: parse_expression(stream)?,
    })
}

pub fn parse_declare_body<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<DeclareBody<'arena>, ParseError> {
    let next = utils::peek(stream)?;

    Ok(match next.kind {
        T![":"] => DeclareBody::ColonDelimited(parse_declare_colon_delimited_body(stream)?),
        _ => DeclareBody::Statement({
            let stmt = parse_statement(stream)?;

            stream.alloc(stmt)
        }),
    })
}

pub fn parse_declare_colon_delimited_body<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<DeclareColonDelimitedBody<'arena>, ParseError> {
    Ok(DeclareColonDelimitedBody {
        colon: utils::expect_span(stream, T![":"])?,
        statements: {
            let mut statements = stream.new_vec();
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
