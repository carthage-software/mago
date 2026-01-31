use crate::T;
use crate::ast::ast::AttributeList;
use crate::ast::ast::HookedProperty;
use crate::ast::ast::Modifier;
use crate::ast::ast::PlainProperty;
use crate::ast::ast::Property;
use crate::ast::ast::PropertyAbstractItem;
use crate::ast::ast::PropertyConcreteItem;
use crate::ast::ast::PropertyHook;
use crate::ast::ast::PropertyHookAbstractBody;
use crate::ast::ast::PropertyHookBody;
use crate::ast::ast::PropertyHookConcreteBody;
use crate::ast::ast::PropertyHookConcreteExpressionBody;
use crate::ast::ast::PropertyHookList;
use crate::ast::ast::PropertyItem;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_property_with_attributes_and_modifiers(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        attributes: Sequence<'arena, AttributeList<'arena>>,
        modifiers: Sequence<'arena, Modifier<'arena>>,
    ) -> Result<Property<'arena>, ParseError> {
        let var = self.maybe_expect_keyword(stream, T!["var"])?;
        let hint = self.parse_optional_type_hint(stream)?;
        let item = self.parse_property_item(stream)?;

        let next = stream.lookahead(0)?.map(|t| t.kind);
        if matches!(next, Some(T!["{"])) {
            return Ok(Property::Hooked(HookedProperty {
                attribute_lists: attributes,
                modifiers,
                var,
                hint,
                item,
                hook_list: self.parse_property_hook_list(stream)?,
            }));
        }

        Ok(Property::Plain(PlainProperty {
            attribute_lists: attributes,
            modifiers,
            var,
            hint,
            items: {
                let mut items = self.new_vec_of(item);
                let mut commas = self.new_vec();
                if matches!(next, Some(T![","])) {
                    commas.push(stream.consume()?);

                    loop {
                        let item = self.parse_property_item(stream)?;
                        items.push(item);

                        match stream.lookahead(0)?.map(|t| t.kind) {
                            Some(T![","]) => {
                                commas.push(stream.consume()?);
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                }

                TokenSeparatedSequence::new(items, commas)
            },
            terminator: self.parse_terminator(stream)?,
        }))
    }

    fn parse_property_item(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<PropertyItem<'arena>, ParseError> {
        Ok(match stream.lookahead(1)?.map(|t| t.kind) {
            Some(T!["="]) => PropertyItem::Concrete(self.parse_property_concrete_item(stream)?),
            _ => PropertyItem::Abstract(self.parse_property_abstract_item(stream)?),
        })
    }

    fn parse_property_abstract_item(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<PropertyAbstractItem<'arena>, ParseError> {
        Ok(PropertyAbstractItem { variable: self.parse_direct_variable(stream)? })
    }

    fn parse_property_concrete_item(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<PropertyConcreteItem<'arena>, ParseError> {
        Ok(PropertyConcreteItem {
            variable: self.parse_direct_variable(stream)?,
            equals: stream.eat(T!["="])?.span,
            value: self.parse_expression(stream)?,
        })
    }

    pub(crate) fn parse_optional_property_hook_list(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<PropertyHookList<'arena>>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["{"]) => Some(self.parse_property_hook_list(stream)?),
            _ => None,
        })
    }

    fn parse_property_hook_list(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<PropertyHookList<'arena>, ParseError> {
        Ok(PropertyHookList {
            left_brace: stream.eat(T!["{"])?.span,
            hooks: {
                let mut hooks = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["}"])) {
                        break;
                    }

                    let hook = self.parse_property_hook(stream)?;
                    hooks.push(hook);
                }

                Sequence::new(hooks)
            },
            right_brace: stream.eat(T!["}"])?.span,
        })
    }

    fn parse_property_hook(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<PropertyHook<'arena>, ParseError> {
        Ok(PropertyHook {
            attribute_lists: self.parse_attribute_list_sequence(stream)?,
            ampersand: if stream.is_at(T!["&"])? { Some(stream.eat(T!["&"])?.span) } else { None },
            modifiers: self.parse_modifier_sequence(stream)?,
            name: self.parse_local_identifier(stream)?,
            parameter_list: self.parse_optional_function_like_parameter_list(stream)?,
            body: self.parse_property_hook_body(stream)?,
        })
    }

    fn parse_property_hook_body(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<PropertyHookBody<'arena>, ParseError> {
        let next = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;

        Ok(match next.kind {
            T![";"] => PropertyHookBody::Abstract(self.parse_property_hook_abstract_body(stream)?),
            T!["{"] | T!["=>"] => PropertyHookBody::Concrete(self.parse_property_hook_concrete_body(stream)?),
            _ => return Err(stream.unexpected(Some(next), T![";", "{", "=>"])),
        })
    }

    fn parse_property_hook_abstract_body(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<PropertyHookAbstractBody, ParseError> {
        Ok(PropertyHookAbstractBody { semicolon: stream.eat(T![";"])?.span })
    }

    fn parse_property_hook_concrete_body(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<PropertyHookConcreteBody<'arena>, ParseError> {
        let next = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;

        Ok(match next.kind {
            T!["{"] => PropertyHookConcreteBody::Block(self.parse_block(stream)?),
            T!["=>"] => {
                PropertyHookConcreteBody::Expression(self.parse_property_hook_concrete_expression_body(stream)?)
            }
            _ => return Err(stream.unexpected(Some(next), T!["{", "=>"])),
        })
    }

    fn parse_property_hook_concrete_expression_body(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<PropertyHookConcreteExpressionBody<'arena>, ParseError> {
        Ok(PropertyHookConcreteExpressionBody {
            arrow: stream.eat(T!["=>"])?.span,
            expression: self.parse_expression(stream)?,
            semicolon: stream.eat(T![";"])?.span,
        })
    }
}
