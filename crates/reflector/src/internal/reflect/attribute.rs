use mago_ast::*;
use mago_reflection::attribute::AttributeArgumentListReflection;
use mago_reflection::attribute::AttributeArgumentReflection;
use mago_reflection::attribute::AttributeReflection;
use mago_reflection::identifier::Name;
use mago_span::*;

use crate::internal::context::Context;

pub fn reflect_attributes<'ast>(
    attribute_lists: &'ast Sequence<AttributeList>,
    context: &'ast mut Context<'_>,
) -> Vec<AttributeReflection> {
    let mut reflections = vec![];

    for attribute_list in attribute_lists.iter() {
        for attribute in attribute_list.attributes.iter() {
            let reflection = AttributeReflection {
                name: {
                    let value = *context.names.get(&attribute.name);

                    Name { value, lower: context.interner.lowered(&value), span: attribute.name.span() }
                },
                arguments: reflect_attribute_arguments(&attribute.arguments, context),
                span: attribute.span(),
            };

            reflections.push(reflection);
        }
    }

    reflections
}

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
                name: {
                    let value = arg.name.value;

                    Name { value, lower: context.interner.lowered(&value), span: arg.name.span }
                },
                value_type_reflection: mago_typing::infere(context.interner, context.source, context.names, &arg.value),
                span: arg.span(),
            },
        });
    }

    Some(AttributeArgumentListReflection { arguments })
}
