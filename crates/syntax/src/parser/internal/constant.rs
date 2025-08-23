use crate::T;
use crate::ast::Sequence;
use crate::ast::ast::*;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::identifier::parse_local_identifier;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_constant_with_attributes<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    attribute_lists: Sequence<'arena, AttributeList<'arena>>,
) -> Result<Constant<'arena>, ParseError> {
    Ok(Constant {
        attribute_lists,
        r#const: utils::expect_keyword(stream, T!["const"])?,
        items: {
            let mut items = stream.new_vec();
            let mut commas = stream.new_vec();
            loop {
                if matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T![";" | "?>"])) {
                    break;
                }

                items.push(parse_constant_item(stream)?);

                match utils::maybe_expect(stream, T![","])? {
                    Some(comma) => commas.push(comma),
                    None => {
                        break;
                    }
                }
            }

            TokenSeparatedSequence::new(items, commas)
        },
        terminator: parse_terminator(stream)?,
    })
}

pub fn parse_constant_item<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<ConstantItem<'arena>, ParseError> {
    Ok(ConstantItem {
        name: parse_local_identifier(stream)?,
        equals: utils::expect_span(stream, T!["="])?,
        value: parse_expression(stream)?,
    })
}
