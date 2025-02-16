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

#[inline]
pub fn parse_property_with_attributes_and_modifiers<'i>(
    stream: &mut TokenStream<'_, 'i>,
    attributes: Sequence<'i, AttributeList<'i>>,
    modifiers: Sequence<'i, Modifier>,
) -> Result<Property<'i>, ParseError> {
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
            let mut items = stream.vec();
            let mut commans = stream.vec();

            items.push(item);
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

#[inline]
pub fn parse_property_item<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<PropertyItem<'i>, ParseError> {
    let next = utils::maybe_peek_nth(stream, 1)?;

    Ok(match next.map(|t| t.kind) {
        Some(T!["="]) => PropertyItem::Concrete(parse_property_concrete_item(stream)?),
        _ => PropertyItem::Abstract(parse_property_abstract_item(stream)?),
    })
}

#[inline]
pub fn parse_property_abstract_item(stream: &mut TokenStream<'_, '_>) -> Result<PropertyAbstractItem, ParseError> {
    Ok(PropertyAbstractItem { variable: parse_direct_variable(stream)? })
}

#[inline]
pub fn parse_property_concrete_item<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<PropertyConcreteItem<'i>, ParseError> {
    Ok(PropertyConcreteItem {
        variable: parse_direct_variable(stream)?,
        equals: utils::expect_span(stream, T!["="])?,
        value: parse_expression(stream)?,
    })
}

#[inline]
pub fn parse_optional_property_hook_list<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<Option<PropertyHookList<'i>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["{"]) => Some(parse_property_hook_list(stream)?),
        _ => None,
    })
}

#[inline]
pub fn parse_property_hook_list<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<PropertyHookList<'i>, ParseError> {
    Ok(PropertyHookList {
        left_brace: utils::expect_span(stream, T!["{"])?,
        hooks: {
            let mut hooks = stream.vec();
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

#[inline]
pub fn parse_property_hook<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<PropertyHook<'i>, ParseError> {
    Ok(PropertyHook {
        attribute_lists: attribute::parse_attribute_list_sequence(stream)?,
        ampersand: utils::maybe_expect(stream, T!["&"])?.map(|t| t.span),
        modifiers: parse_modifier_sequence(stream)?,
        name: identifier::parse_local_identifier(stream)?,
        parameters: parameter::parse_optional_function_like_parameter_list(stream)?,
        body: parse_property_hook_body(stream)?,
    })
}

#[inline]
pub fn parse_property_hook_body<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<PropertyHookBody<'i>, ParseError> {
    let next = utils::peek(stream)?;

    Ok(match next.kind {
        T![";"] => PropertyHookBody::Abstract(parse_property_hook_abstract_body(stream)?),
        T!["{"] | T!["=>"] => PropertyHookBody::Concrete(parse_property_hook_concrete_body(stream)?),
        _ => return Err(utils::unexpected(stream, Some(next), T![";", "{", "=>"])),
    })
}

#[inline]
pub fn parse_property_hook_abstract_body(
    stream: &mut TokenStream<'_, '_>,
) -> Result<PropertyHookAbstractBody, ParseError> {
    Ok(PropertyHookAbstractBody { semicolon: utils::expect_span(stream, T![";"])? })
}

#[inline]
pub fn parse_property_hook_concrete_body<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<PropertyHookConcreteBody<'i>, ParseError> {
    let next = utils::peek(stream)?;

    Ok(match next.kind {
        T!["{"] => PropertyHookConcreteBody::Block(parse_block(stream)?),
        T!["=>"] => PropertyHookConcreteBody::Expression(parse_property_hook_concrete_expression_body(stream)?),
        _ => return Err(utils::unexpected(stream, Some(next), T!["{", "=>"])),
    })
}

#[inline]
pub fn parse_property_hook_concrete_expression_body<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<PropertyHookConcreteExpressionBody<'i>, ParseError> {
    Ok(PropertyHookConcreteExpressionBody {
        arrow: utils::expect_span(stream, T!["=>"])?,
        expression: expression::parse_expression(stream)?,
        semicolon: utils::expect_span(stream, T![";"])?,
    })
}
