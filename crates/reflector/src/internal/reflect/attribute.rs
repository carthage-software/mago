use fennec_ast::*;
use fennec_reflection::attribute::AttributeArgumentListReflection;
use fennec_reflection::attribute::AttributeArgumentReflection;
use fennec_reflection::attribute::AttributeReflection;
use fennec_reflection::identifier::Name;
use fennec_span::*;

use crate::internal::context::Context;

pub fn reflect_attributes<'i, 'ast>(
    attribute_lists: &'ast Sequence<AttributeList>,
    context: &'ast mut Context<'i>,
) -> Vec<AttributeReflection> {
    let mut reflections = vec![];

    for attribute_list in attribute_lists.iter() {
        for attribute in attribute_list.attributes.iter() {
            let reflection = AttributeReflection {
                name: Name { value: context.semantics.names.get(&attribute.name), span: attribute.name.span() },
                arguments: reflect_attribute_arguments(&attribute.arguments, context),
                span: attribute.span(),
            };

            reflections.push(reflection);
        }
    }

    reflections
}

pub fn reflect_attribute_arguments<'i, 'ast>(
    argument_list: &'ast Option<ArgumentList>,
    context: &'ast mut Context<'i>,
) -> Option<AttributeArgumentListReflection> {
    let Some(argument_list) = argument_list else {
        return None;
    };

    let mut arguments = vec![];
    for argument in argument_list.arguments.iter() {
        arguments.push(match &argument {
            Argument::Positional(arg) => AttributeArgumentReflection::Positional {
                value_type_reflection: fennec_inference::infere(&context.interner, &context.semantics, &arg.value),
                span: arg.span(),
            },
            Argument::Named(arg) => AttributeArgumentReflection::Named {
                name: Name { value: arg.name.value, span: arg.name.span },
                value_type_reflection: fennec_inference::infere(&context.interner, &context.semantics, &arg.value),
                span: arg.span(),
            },
        });
    }

    Some(AttributeArgumentListReflection { arguments })
}
