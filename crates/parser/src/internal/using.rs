use mago_ast::ast::*;
use mago_ast::sequence::TokenSeparatedSequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::statement::parse_statement;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;
use crate::internal::variable::parse_direct_variable;

pub fn parse_using(stream: &mut TokenStream<'_, '_>) -> Result<Using, ParseError> {
    let using = utils::expect_keyword(stream, T!["using"])?;
    let left_parenthesis = utils::expect_span(stream, T!["("])?;

    let mut items = vec![];
    let mut commas = vec![];
    loop {
        if matches!(utils::peek(stream)?.kind, T![")"]) {
            break;
        }

        items.push(parse_using_item(stream)?);

        match utils::peek(stream)?.kind {
            T![","] => {
                commas.push(utils::expect_any(stream)?);
            }
            _ => {
                break;
            }
        }
    }

    let right_parenthesis = utils::expect_span(stream, T![")"])?;
    let statement = parse_statement(stream)?;

    Ok(Using {
        using,
        left_parenthesis,
        items: TokenSeparatedSequence::new(items, commas),
        right_parenthesis,
        statement: Box::new(statement),
    })
}

#[inline]
fn parse_using_item(stream: &mut TokenStream<'_, '_>) -> Result<UsingItem, ParseError> {
    let direct_variable = parse_direct_variable(stream)?;

    match utils::peek(stream)?.kind {
        T!["="] => {
            let equals = utils::expect_any(stream)?;
            let expression = parse_expression(stream)?;
            Ok(UsingItem::Concrete(direct_variable, equals.span, expression))
        }
        _ => Ok(UsingItem::Abstract(direct_variable)),
    }
}
