use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;
use crate::parser::internal::variable::parse_direct_variable;

pub fn parse_static<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Static<'arena>, ParseError> {
    let r#static = utils::expect_keyword(stream, T!["static"])?;
    let items = {
        let mut items = stream.new_vec();
        let mut commas = stream.new_vec();

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

pub fn parse_static_item<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<StaticItem<'arena>, ParseError> {
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
