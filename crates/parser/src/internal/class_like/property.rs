use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_ast::sequence::TokenSeparatedSequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::attribute;
use crate::internal::block::parse_block;
use crate::internal::expression;
use crate::internal::expression::parse_expression;
use crate::internal::function_like::parameter;
use crate::internal::identifier;
use crate::internal::modifier::parse_modifier_sequence;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::type_hint::parse_optional_type_hint;
use crate::internal::utils;
use crate::internal::variable::parse_direct_variable;

pub fn parse_property_with_attributes_and_modifiers(
    stream: &mut TokenStream<'_, '_>,
    attributes: Sequence<AttributeList>,
    modifiers: Sequence<Modifier>,
) -> Result<Property, ParseError> {
    let var = utils::maybe_expect_keyword(stream, T!["var"])?;
    let hint = parse_optional_type_hint(stream)?;
    let item = parse_property_item(stream)?;

    let next = utils::peek(stream)?.kind;
    if matches!(next, T!["{"]) {
        return Ok(Property::Hooked(HookedProperty {
            attribute_lists: attributes,
            modifiers,
            var,
            hint,
            item,
            hooks: parse_property_hook_list(stream)?,
        }));
    }

    Ok(Property::Plain(PlainProperty {
        attribute_lists: attributes,
        modifiers,
        var,
        hint,
        items: {
            let mut items = vec![item];
            let mut commans = Vec::new();
            if matches!(next, T![","]) {
                commans.push(utils::expect_any(stream)?);

                loop {
                    let item = parse_property_item(stream)?;
                    items.push(item);

                    match utils::maybe_expect(stream, T![","])? {
                        Some(comma) => {
                            commans.push(comma);
                        }
                        None => {
                            break;
                        }
                    }
                }
            }

            TokenSeparatedSequence::new(items, commans)
        },
        terminator: parse_terminator(stream)?,
    }))
}

pub fn parse_property_item(stream: &mut TokenStream<'_, '_>) -> Result<PropertyItem, ParseError> {
    let next = utils::maybe_peek_nth(stream, 1)?;

    Ok(match next.map(|t| t.kind) {
        Some(T!["="]) => PropertyItem::Concrete(parse_property_concrete_item(stream)?),
        _ => PropertyItem::Abstract(parse_property_abstract_item(stream)?),
    })
}

pub fn parse_property_abstract_item(stream: &mut TokenStream<'_, '_>) -> Result<PropertyAbstractItem, ParseError> {
    Ok(PropertyAbstractItem { variable: parse_direct_variable(stream)? })
}

pub fn parse_property_concrete_item(stream: &mut TokenStream<'_, '_>) -> Result<PropertyConcreteItem, ParseError> {
    Ok(PropertyConcreteItem {
        variable: parse_direct_variable(stream)?,
        equals: utils::expect_span(stream, T!["="])?,
        value: parse_expression(stream)?,
    })
}

pub fn parse_optional_property_hook_list(
    stream: &mut TokenStream<'_, '_>,
) -> Result<Option<PropertyHookList>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["{"]) => Some(parse_property_hook_list(stream)?),
        _ => None,
    })
}

pub fn parse_property_hook_list(stream: &mut TokenStream<'_, '_>) -> Result<PropertyHookList, ParseError> {
    Ok(PropertyHookList {
        left_brace: utils::expect_span(stream, T!["{"])?,
        hooks: {
            let mut hooks = Vec::new();
            loop {
                let token = utils::peek(stream)?;
                if T!["}"] == token.kind {
                    break;
                }

                let hook = parse_property_hook(stream)?;
                hooks.push(hook);
            }

            Sequence::new(hooks)
        },
        right_brace: utils::expect_span(stream, T!["}"])?,
    })
}

pub fn parse_property_hook(stream: &mut TokenStream<'_, '_>) -> Result<PropertyHook, ParseError> {
    Ok(PropertyHook {
        attribute_lists: attribute::parse_attribute_list_sequence(stream)?,
        ampersand: utils::maybe_expect(stream, T!["&"])?.map(|t| t.span),
        modifiers: parse_modifier_sequence(stream)?,
        name: identifier::parse_local_identifier(stream)?,
        parameters: parameter::parse_optional_function_like_parameter_list(stream)?,
        body: parse_property_hook_body(stream)?,
    })
}

pub fn parse_property_hook_body(stream: &mut TokenStream<'_, '_>) -> Result<PropertyHookBody, ParseError> {
    let next = utils::peek(stream)?;

    Ok(match next.kind {
        T![";"] => PropertyHookBody::Abstract(parse_property_hook_abstract_body(stream)?),
        T!["{"] | T!["=>"] => PropertyHookBody::Concrete(parse_property_hook_concrete_body(stream)?),
        _ => return Err(utils::unexpected(stream, Some(next), T![";", "{", "=>"])),
    })
}

pub fn parse_property_hook_abstract_body(
    stream: &mut TokenStream<'_, '_>,
) -> Result<PropertyHookAbstractBody, ParseError> {
    Ok(PropertyHookAbstractBody { semicolon: utils::expect_span(stream, T![";"])? })
}

pub fn parse_property_hook_concrete_body(
    stream: &mut TokenStream<'_, '_>,
) -> Result<PropertyHookConcreteBody, ParseError> {
    let next = utils::peek(stream)?;

    Ok(match next.kind {
        T!["{"] => PropertyHookConcreteBody::Block(parse_block(stream)?),
        T!["=>"] => PropertyHookConcreteBody::Expression(parse_property_hook_concrete_expression_body(stream)?),
        _ => return Err(utils::unexpected(stream, Some(next), T!["{", "=>"])),
    })
}

pub fn parse_property_hook_concrete_expression_body(
    stream: &mut TokenStream<'_, '_>,
) -> Result<PropertyHookConcreteExpressionBody, ParseError> {
    Ok(PropertyHookConcreteExpressionBody {
        arrow: utils::expect_span(stream, T!["=>"])?,
        expression: expression::parse_expression(stream)?,
        semicolon: utils::expect_span(stream, T![";"])?,
    })
}
