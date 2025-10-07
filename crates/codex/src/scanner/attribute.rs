use mago_atom::Atom;
use mago_atom::atom;
use mago_names::scope::NamespaceScope;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::flags::attribute::AttributeFlags;
use crate::metadata::attribute::AttributeMetadata;
use crate::scanner::Context;
use crate::scanner::inference::infer;

#[inline]
pub fn scan_attribute_lists<'ctx, 'arena>(
    attribute_lists: &'arena Sequence<'arena, AttributeList<'arena>>,
    context: &mut Context<'ctx, 'arena>,
) -> Vec<AttributeMetadata> {
    let mut metadata = vec![];

    for attribute_list in attribute_lists.iter() {
        for attribute in attribute_list.attributes.iter() {
            metadata.push(AttributeMetadata {
                name: atom(context.resolved_names.get(&attribute.name)),
                span: attribute.span(),
            });
        }
    }

    metadata
}

#[inline]
pub fn get_attribute_flags<'ctx, 'arena>(
    class_like_name: Atom,
    attribute_lists: &'arena Sequence<'arena, AttributeList<'arena>>,
    context: &mut Context<'ctx, 'arena>,
    scope: &NamespaceScope,
) -> Option<AttributeFlags> {
    if class_like_name.eq_ignore_ascii_case("Attribute") {
        return Some(AttributeFlags::TARGET_CLASS);
    }

    for attribute in attribute_lists.iter().flat_map(|list| list.attributes.iter()) {
        let attribute_name = context.resolved_names.get(&attribute.name);
        if !attribute_name.eq_ignore_ascii_case("Attribute") {
            continue;
        }

        let Some(first_argument) =
            attribute.argument_list.as_ref().and_then(|argument_list| argument_list.arguments.first())
        else {
            // No target specified means all targets
            return Some(AttributeFlags::TARGET_ALL);
        };

        let inferred_type = infer(context, scope, first_argument.value());
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
