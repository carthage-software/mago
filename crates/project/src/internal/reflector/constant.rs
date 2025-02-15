use mago_ast::*;
use mago_reflection::constant::ConstantReflection;
use mago_reflection::identifier::Name;
use mago_span::*;

use crate::internal::context::Context;
use crate::internal::reflector::attribute::reflect_attributes;

#[inline]
pub fn reflect_constant(constant: &Constant, context: &mut Context<'_>) -> Vec<ConstantReflection> {
    let attribute_reflections = reflect_attributes(&constant.attribute_lists, context);

    let mut reflections = vec![];
    for item in constant.items.iter() {
        let name = context.names.get(&item.name);

        reflections.push(ConstantReflection {
            attribute_reflections: attribute_reflections.clone(),
            name: Name::new(*name, item.name.span),
            type_reflection: mago_typing::infere(context.interner, context.source, context.names, &item.value),
            item_span: item.span(),
            definition_span: constant.span(),
            is_populated: false,
            issues: Default::default(),
        });
    }

    reflections
}

#[inline]
pub fn reflect_defined_constant(define: &FunctionCall, context: &mut Context<'_>) -> Option<ConstantReflection> {
    let Expression::Identifier(identifier) = define.function.as_ref() else {
        return None;
    };

    let function_name = context.interner.lookup(&identifier.value());
    if function_name != "define" {
        return None;
    }

    let arguments = define.argument_list.arguments.as_slice();
    if arguments.len() != 2 {
        return None;
    }

    let Expression::Literal(Literal::String(name_string)) = arguments[0].value() else {
        return None;
    };

    let name_span = name_string.span();
    let name_string = context.interner.lookup(&name_string.value);
    let name = name_string[1..name_string.len() - 1].to_owned();
    let name = context.interner.intern(name);

    Some(ConstantReflection {
        attribute_reflections: Default::default(),
        name: Name::new(name, name_span),
        type_reflection: mago_typing::infere(context.interner, context.source, context.names, arguments[1].value()),
        item_span: define.span(),
        definition_span: define.span(),
        is_populated: false,
        issues: Default::default(),
    })
}
