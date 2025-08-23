use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::attribute;
use crate::parser::internal::block::parse_block;
use crate::parser::internal::expression;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::function_like::parameter;
use crate::parser::internal::identifier;
use crate::parser::internal::modifier::parse_modifier_sequence;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::type_hint::parse_optional_type_hint;
use crate::parser::internal::utils;
use crate::parser::internal::variable::parse_direct_variable;

pub fn parse_property_with_attributes_and_modifiers<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    attributes: Sequence<'arena, AttributeList<'arena>>,
    modifiers: Sequence<'arena, Modifier<'arena>>,
) -> Result<Property<'arena>, ParseError> {
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
            hook_list: parse_property_hook_list(stream)?,
        }));
    }

    Ok(Property::Plain(PlainProperty {
        attribute_lists: attributes,
        modifiers,
        var,
        hint,
        items: {
            let mut items = stream.new_vec_of(item);
            let mut commas = stream.new_vec();
            if matches!(next, T![","]) {
                commas.push(utils::expect_any(stream)?);

                loop {
                    let item = parse_property_item(stream)?;
                    items.push(item);

                    match utils::maybe_expect(stream, T![","])? {
                        Some(comma) => {
                            commas.push(comma);
                        }
                        None => {
                            break;
                        }
                    }
                }
            }

            TokenSeparatedSequence::new(items, commas)
        },
        terminator: parse_terminator(stream)?,
    }))
}

pub fn parse_property_item<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<PropertyItem<'arena>, ParseError> {
    let next = utils::maybe_peek_nth(stream, 1)?;

    Ok(match next.map(|t| t.kind) {
        Some(T!["="]) => PropertyItem::Concrete(parse_property_concrete_item(stream)?),
        _ => PropertyItem::Abstract(parse_property_abstract_item(stream)?),
    })
}

pub fn parse_property_abstract_item<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<PropertyAbstractItem<'arena>, ParseError> {
    Ok(PropertyAbstractItem { variable: parse_direct_variable(stream)? })
}

pub fn parse_property_concrete_item<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<PropertyConcreteItem<'arena>, ParseError> {
    Ok(PropertyConcreteItem {
        variable: parse_direct_variable(stream)?,
        equals: utils::expect_span(stream, T!["="])?,
        value: parse_expression(stream)?,
    })
}

pub fn parse_optional_property_hook_list<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<PropertyHookList<'arena>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["{"]) => Some(parse_property_hook_list(stream)?),
        _ => None,
    })
}

pub fn parse_property_hook_list<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<PropertyHookList<'arena>, ParseError> {
    Ok(PropertyHookList {
        left_brace: utils::expect_span(stream, T!["{"])?,
        hooks: {
            let mut hooks = stream.new_vec();
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

pub fn parse_property_hook<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<PropertyHook<'arena>, ParseError> {
    Ok(PropertyHook {
        attribute_lists: attribute::parse_attribute_list_sequence(stream)?,
        ampersand: utils::maybe_expect(stream, T!["&"])?.map(|t| t.span),
        modifiers: parse_modifier_sequence(stream)?,
        name: identifier::parse_local_identifier(stream)?,
        parameters: parameter::parse_optional_function_like_parameter_list(stream)?,
        body: parse_property_hook_body(stream)?,
    })
}

pub fn parse_property_hook_body<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<PropertyHookBody<'arena>, ParseError> {
    let next = utils::peek(stream)?;

    Ok(match next.kind {
        T![";"] => PropertyHookBody::Abstract(parse_property_hook_abstract_body(stream)?),
        T!["{"] | T!["=>"] => PropertyHookBody::Concrete(parse_property_hook_concrete_body(stream)?),
        _ => return Err(utils::unexpected(stream, Some(next), T![";", "{", "=>"])),
    })
}

pub fn parse_property_hook_abstract_body<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<PropertyHookAbstractBody, ParseError> {
    Ok(PropertyHookAbstractBody { semicolon: utils::expect_span(stream, T![";"])? })
}

pub fn parse_property_hook_concrete_body<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<PropertyHookConcreteBody<'arena>, ParseError> {
    let next = utils::peek(stream)?;

    Ok(match next.kind {
        T!["{"] => PropertyHookConcreteBody::Block(parse_block(stream)?),
        T!["=>"] => PropertyHookConcreteBody::Expression(parse_property_hook_concrete_expression_body(stream)?),
        _ => return Err(utils::unexpected(stream, Some(next), T!["{", "=>"])),
    })
}

pub fn parse_property_hook_concrete_expression_body<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<PropertyHookConcreteExpressionBody<'arena>, ParseError> {
    Ok(PropertyHookConcreteExpressionBody {
        arrow: utils::expect_span(stream, T!["=>"])?,
        expression: expression::parse_expression(stream)?,
        semicolon: utils::expect_span(stream, T![";"])?,
    })
}
