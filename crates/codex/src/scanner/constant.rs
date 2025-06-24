use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::lower_constant_name;
use crate::metadata::constant::ConstantMetadata;
use crate::scanner::Context;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::docblock::ConstantDocblockComment;
use crate::scanner::inference::infer;

#[inline]
pub fn scan_constant(constant: &Constant, context: &mut Context<'_>) -> Vec<ConstantMetadata> {
    let attributes = scan_attribute_lists(&constant.attribute_lists, context);
    let docblock = ConstantDocblockComment::create(context, constant);

    constant
        .items
        .iter()
        .map(|item| {
            let name = lower_constant_name(context.interner, context.resolved_names.get(&item.name));

            let metadata = ConstantMetadata::new(name, item.span())
                .with_inferred_type(infer(context.interner, context.resolved_names, &item.value))
                .with_attributes(attributes.clone());

            if let Some(ref docblock) = docblock {
                metadata.with_is_deprecated(docblock.is_deprecated).with_is_internal(docblock.is_internal)
            } else {
                metadata
            }
        })
        .collect()
}

#[inline]
pub fn scan_defined_constant(define: &FunctionCall, context: &mut Context<'_>) -> Option<ConstantMetadata> {
    let Expression::Identifier(identifier) = define.function.as_ref() else {
        return None;
    };

    let function_name = context.interner.lookup(identifier.value());
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

    let name = context.interner.intern(name_string.value.as_deref()?);
    let name = lower_constant_name(context.interner, &name);

    Some(
        ConstantMetadata::new(name, define.span())
            .with_inferred_type(infer(context.interner, context.resolved_names, arguments[1].value()))
            .with_is_deprecated(false)
            .with_is_internal(false),
    )
}
