use std::str::FromStr;

use mago_php_version::PHPVersion;
use mago_span::HasPosition;
use mago_syntax::ast::Argument;
use mago_syntax::ast::Attribute;
use mago_syntax::ast::AttributeList;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Literal;
use mago_syntax::ast::Sequence;

use crate::internal::context::Context;

pub mod attribute;
pub mod class_like;
pub mod constant;
pub mod function_like;
pub mod r#type;

#[inline]
fn should_reflect_element<'a>(context: &'a mut Context<'_>, attribute_lists: &'a Sequence<AttributeList>) -> bool {
    for attribute_list in attribute_lists.as_slice() {
        for attribute in attribute_list.attributes.as_slice() {
            let name = context.get_name(&attribute.name.position());
            if name == "JetBrains\\PhpStorm\\Internal\\PhpStormStubsElementAvailable" {
                let (from, to) = get_availability_range(context, attribute);

                if let Some(from) = from {
                    if context.version < &from {
                        return false;
                    }
                }

                if let Some(to) = to {
                    if context.version > &to {
                        return false;
                    }
                }
            }
        }
    }

    true
}

#[inline]
fn get_availability_range<'a>(
    context: &'a mut Context<'_>,
    attribute: &'a Attribute,
) -> (Option<PHPVersion>, Option<PHPVersion>) {
    let mut from = None;
    let mut to = None;

    let Some(argument_list) = attribute.arguments.as_ref() else {
        return (from, to);
    };

    let Some(first_argument) = argument_list.arguments.get(0) else {
        return (from, to);
    };

    match first_argument {
        Argument::Positional(positional_argument) => {
            from = get_php_version_from_expression(context, &positional_argument.value);
        }
        Argument::Named(named) => {
            let name = context.interner.lookup(&named.name.value);
            if name == "from" {
                from = get_php_version_from_expression(context, &named.value);
            } else if name == "to" {
                to = get_php_version_from_expression(context, &named.value);
            }
        }
    }

    if from.is_some() && to.is_none() {
        let Some(second_argument) = argument_list.arguments.get(1) else {
            return (from, to);
        };

        to = get_php_version_from_expression(context, second_argument.value());
    } else if from.is_none() && to.is_some() {
        let Some(second_argument) = argument_list.arguments.get(1) else {
            return (from, to);
        };

        from = get_php_version_from_expression(context, second_argument.value());
    }

    (from, to)
}

#[inline]
fn get_php_version_from_expression<'a>(context: &'a mut Context<'_>, expression: &'a Expression) -> Option<PHPVersion> {
    let Expression::Literal(Literal::String(literal_string)) = expression else {
        return None;
    };

    let string = context.interner.lookup(&literal_string.value);

    PHPVersion::from_str(&string[1..string.len() - 1]).ok()
}
