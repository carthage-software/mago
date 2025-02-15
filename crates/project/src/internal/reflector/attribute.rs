use mago_ast::*;
use mago_reflection::attribute::AttributeArgumentListReflection;
use mago_reflection::attribute::AttributeArgumentReflection;
use mago_reflection::attribute::AttributeReflection;
use mago_reflection::identifier::Name;
use mago_span::*;

use crate::internal::context::Context;

#[inline]
pub fn reflect_attributes<'ast>(
    attribute_lists: &'ast Sequence<AttributeList>,
    context: &'ast mut Context<'_>,
) -> Vec<AttributeReflection> {
    let mut reflections = vec![];

    for attribute_list in attribute_lists.iter() {
        for attribute in attribute_list.attributes.iter() {
            let reflection = AttributeReflection {
                name: Name::new(*context.names.get(&attribute.name), attribute.name.span()),
                arguments: reflect_attribute_arguments(&attribute.arguments, context),
                span: attribute.span(),
            };

            reflections.push(reflection);
        }
    }

    reflections
}

#[inline]
pub fn reflect_attribute_arguments<'ast>(
    argument_list: &'ast Option<ArgumentList>,
    context: &'ast mut Context<'_>,
) -> Option<AttributeArgumentListReflection> {
    let Some(argument_list) = argument_list else {
        return None;
    };

    let mut arguments = vec![];
    for argument in argument_list.arguments.iter() {
        arguments.push(match &argument {
            Argument::Positional(arg) => AttributeArgumentReflection::Positional {
                value_type_reflection: mago_typing::infere(context.interner, context.source, context.names, &arg.value),
                span: arg.span(),
            },
            Argument::Named(arg) => AttributeArgumentReflection::Named {
                name: Name::new(arg.name.value, arg.name.span),
                value_type_reflection: mago_typing::infere(context.interner, context.source, context.names, &arg.value),
                span: arg.span(),
            },
        });
    }

    Some(AttributeArgumentListReflection { arguments })
}
