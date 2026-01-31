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

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_property_with_attributes_and_modifiers(
        &mut self,
        attributes: Sequence<'arena, AttributeList<'arena>>,
        modifiers: Sequence<'arena, Modifier<'arena>>,
    ) -> Result<Property<'arena>, ParseError> {
        let var = self.maybe_expect_keyword(T!["var"])?;
        let hint = self.parse_optional_type_hint()?;
        let item = self.parse_property_item()?;

        let next = self.stream.lookahead(0)?.map(|t| t.kind);
        if matches!(next, Some(T!["{"])) {
            return Ok(Property::Hooked(HookedProperty {
                attribute_lists: attributes,
                modifiers,
                var,
                hint,
                item,
                hook_list: self.parse_property_hook_list()?,
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
                    commas.push(self.stream.consume()?);

                    loop {
                        let item = self.parse_property_item()?;
                        items.push(item);

                        match self.stream.lookahead(0)?.map(|t| t.kind) {
                            Some(T![","]) => {
                                commas.push(self.stream.consume()?);
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                }

                TokenSeparatedSequence::new(items, commas)
            },
            terminator: self.parse_terminator()?,
        }))
    }

    fn parse_property_item(&mut self) -> Result<PropertyItem<'arena>, ParseError> {
        Ok(match self.stream.lookahead(1)?.map(|t| t.kind) {
            Some(T!["="]) => PropertyItem::Concrete(self.parse_property_concrete_item()?),
            _ => PropertyItem::Abstract(self.parse_property_abstract_item()?),
        })
    }

    fn parse_property_abstract_item(&mut self) -> Result<PropertyAbstractItem<'arena>, ParseError> {
        Ok(PropertyAbstractItem { variable: self.parse_direct_variable()? })
    }

    fn parse_property_concrete_item(&mut self) -> Result<PropertyConcreteItem<'arena>, ParseError> {
        Ok(PropertyConcreteItem {
            variable: self.parse_direct_variable()?,
            equals: self.stream.eat(T!["="])?.span,
            value: self.parse_expression()?,
        })
    }

    pub(crate) fn parse_optional_property_hook_list(&mut self) -> Result<Option<PropertyHookList<'arena>>, ParseError> {
        Ok(match self.stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["{"]) => Some(self.parse_property_hook_list()?),
            _ => None,
        })
    }

    fn parse_property_hook_list(&mut self) -> Result<PropertyHookList<'arena>, ParseError> {
        Ok(PropertyHookList {
            left_brace: self.stream.eat(T!["{"])?.span,
            hooks: {
                let mut hooks = self.new_vec();
                loop {
                    if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T!["}"])) {
                        break;
                    }

                    let hook = self.parse_property_hook()?;
                    hooks.push(hook);
                }

                Sequence::new(hooks)
            },
            right_brace: self.stream.eat(T!["}"])?.span,
        })
    }

    fn parse_property_hook(&mut self) -> Result<PropertyHook<'arena>, ParseError> {
        Ok(PropertyHook {
            attribute_lists: self.parse_attribute_list_sequence()?,
            ampersand: if self.stream.is_at(T!["&"])? { Some(self.stream.eat(T!["&"])?.span) } else { None },
            modifiers: self.parse_modifier_sequence()?,
            name: self.parse_local_identifier()?,
            parameter_list: self.parse_optional_function_like_parameter_list()?,
            body: self.parse_property_hook_body()?,
        })
    }

    fn parse_property_hook_body(&mut self) -> Result<PropertyHookBody<'arena>, ParseError> {
        let next = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;

        Ok(match next.kind {
            T![";"] => PropertyHookBody::Abstract(self.parse_property_hook_abstract_body()?),
            T!["{"] | T!["=>"] => PropertyHookBody::Concrete(self.parse_property_hook_concrete_body()?),
            _ => return Err(self.stream.unexpected(Some(next), T![";", "{", "=>"])),
        })
    }

    fn parse_property_hook_abstract_body(&mut self) -> Result<PropertyHookAbstractBody, ParseError> {
        Ok(PropertyHookAbstractBody { semicolon: self.stream.eat(T![";"])?.span })
    }

    fn parse_property_hook_concrete_body(&mut self) -> Result<PropertyHookConcreteBody<'arena>, ParseError> {
        let next = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;

        Ok(match next.kind {
            T!["{"] => PropertyHookConcreteBody::Block(self.parse_block()?),
            T!["=>"] => PropertyHookConcreteBody::Expression(self.parse_property_hook_concrete_expression_body()?),
            _ => return Err(self.stream.unexpected(Some(next), T!["{", "=>"])),
        })
    }

    fn parse_property_hook_concrete_expression_body(
        &mut self,
    ) -> Result<PropertyHookConcreteExpressionBody<'arena>, ParseError> {
        Ok(PropertyHookConcreteExpressionBody {
            arrow: self.stream.eat(T!["=>"])?.span,
            expression: self.parse_expression()?,
            semicolon: self.stream.eat(T![";"])?.span,
        })
    }
}
