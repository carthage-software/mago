use mago_ast::*;
use mago_interner::StringIdentifier;
use mago_reflection::constant::ConstantReflection;
use mago_reflection::identifier::Name;
use mago_span::*;

use crate::internal::context::Context;

pub fn reflect_constant(constant: &Constant, context: &mut Context<'_>) -> Vec<ConstantReflection> {
    let mut reflections = vec![];
    for item in constant.items.iter() {
        let name = context.names.get(&item.name);
        let lower = get_lower_name(context, name);

        reflections.push(ConstantReflection {
            name: Name::new(*name, lower, item.name.span),
            type_reflection: mago_typing::infere(context.interner, context.source, context.names, &item.value),
            item_span: item.span(),
            definition_span: constant.span(),
            is_populated: false,
        });
    }

    reflections
}

pub fn reflect_defined_constant(define: &FunctionCall, context: &mut Context<'_>) -> Option<ConstantReflection> {
    let Expression::Identifier(identifier) = define.function.as_ref() else {
        return None;
    };

    let function_name = context.interner.lookup(&identifier.value());
    if function_name != "define" {
        return None;
    }

    let arguments = define.arguments.arguments.as_slice();
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
    let lower = get_lower_name(context, &name);

    Some(ConstantReflection {
        name: Name::new(name, lower, name_span),
        type_reflection: mago_typing::infere(context.interner, context.source, context.names, arguments[1].value()),
        item_span: define.span(),
        definition_span: define.span(),
        is_populated: false,
    })
}

fn get_lower_name(context: &Context, name: &StringIdentifier) -> StringIdentifier {
    let name = context.interner.lookup(name);

    let mut parts: Vec<_> = name.split('\\').map(str::to_owned).collect();
    let total_parts = parts.len();
    if total_parts > 1 {
        parts = parts
            .into_iter()
            .enumerate()
            .map(|(i, part)| if i < total_parts - 1 { part.to_ascii_lowercase() } else { part })
            .collect::<Vec<_>>();
    }

    context.interner.intern(parts.join("\\"))
}
