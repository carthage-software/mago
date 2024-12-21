use mago_ast::*;
use mago_reflection::constant::ConstantReflection;
use mago_reflection::identifier::Name;
use mago_span::*;

use crate::internal::context::Context;

pub fn reflect_constant<'ast>(constant: &'ast Constant, context: &'ast mut Context<'_>) -> Vec<ConstantReflection> {
    let mut reflections = vec![];
    for item in constant.items.iter() {
        let name = context.names.get(&item.name);

        reflections.push(ConstantReflection {
            name: Name::new(*name, item.name.span),
            type_reflection: mago_typing::infere(context.interner, context.source, context.names, &item.value),
            item_span: item.span(),
            definition_span: constant.span(),
            is_populated: false,
        });
    }

    reflections
}
