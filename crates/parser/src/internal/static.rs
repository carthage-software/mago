use mago_ast::ast::*;
use mago_ast::sequence::TokenSeparatedSequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;
use crate::internal::variable::parse_direct_variable;

pub fn parse_static<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Static<'i>, ParseError> {
    let r#static = utils::expect_keyword(stream, T!["static"])?;
    let items = {
        let mut items = stream.vec();
        let mut commas = stream.vec();

        loop {
            if matches!(utils::peek(stream)?.kind, T!["?>" | ";"]) {
                break;
            }

            items.push(parse_static_item(stream)?);

            match utils::peek(stream)?.kind {
                T![","] => {
                    commas.push(utils::expect_any(stream)?);
                }
                _ => {
                    break;
                }
            }
        }

        TokenSeparatedSequence::new(items, commas)
    };
    let terminator = parse_terminator(stream)?;

    Ok(Static { r#static, items, terminator })
}

pub fn parse_static_item<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<StaticItem<'i>, ParseError> {
    let variable = parse_direct_variable(stream)?;

    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["="]) => {
            let equals = utils::expect_span(stream, T!["="])?;
            let value = parse_expression(stream)?;

            StaticItem::Concrete(StaticConcreteItem { variable, equals, value })
        }
        _ => StaticItem::Abstract(StaticAbstractItem { variable }),
    })
}
