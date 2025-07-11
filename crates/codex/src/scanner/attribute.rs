use mago_interner::StringIdentifier;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::flags::attribute::AttributeFlags;
use crate::metadata::attribute::AttributeMetadata;
use crate::scanner::Context;
use crate::scanner::argument::scan_argument_list;
use crate::scanner::inference::infer;

#[inline]
pub fn scan_attribute_lists<'ast>(
    attribute_lists: &'ast Sequence<AttributeList>,
    context: &'ast mut Context<'_>,
) -> Vec<AttributeMetadata> {
    let mut metadata = vec![];

    for attribute_list in attribute_lists.iter() {
        for attribute in attribute_list.attributes.iter() {
            let name = *context.resolved_names.get(&attribute.name);
            let name_span = attribute.name.span();
            let span = attribute.span();

            metadata.push(AttributeMetadata::new(name, name_span, span).with_arguments(
                match &attribute.argument_list {
                    Some(argument_list) => scan_argument_list(argument_list, context),
                    None => vec![],
                },
            ));
        }
    }

    metadata
}

#[inline]
pub fn get_attribute_flags<'ast>(
    class_like_name: StringIdentifier,
    attribute_lists: &'ast Sequence<AttributeList>,
    context: &'ast mut Context<'_>,
) -> Option<AttributeFlags> {
    let class_like_name_str = context.interner.lookup(&class_like_name);
    if class_like_name_str.eq_ignore_ascii_case("Attribute") {
        return Some(AttributeFlags::TARGET_CLASS);
    }

    for attribute in attribute_lists.iter().flat_map(|list| list.attributes.iter()) {
        let attribute_name = context.interner.lookup(context.resolved_names.get(&attribute.name));
        if !attribute_name.eq_ignore_ascii_case("Attribute") {
            continue;
        }

        let Some(first_argument) =
            attribute.argument_list.as_ref().and_then(|argument_list| argument_list.arguments.first())
        else {
            // No target specified means all targets
            return Some(AttributeFlags::TARGET_ALL);
        };

        let inferred_type = infer(context.interner, context.resolved_names, first_argument.value());
        let bits = inferred_type.and_then(|i| i.get_single_literal_int_value()).and_then(|value| {
            if !(0..=255).contains(&value) {
                return None;
            }

            Some(value as u8)
        });

        let attributes = if let Some(bits) = bits {
            AttributeFlags::from_bits(bits)
        } else {
            // Unable to infer the target, allow all targets + repeatable
            Some(AttributeFlags::all())
        };

        return attributes;
    }

    None
}
