use mago_allocator::Arena;
use mago_names::scope::NamespaceScope;
use mago_span::HasSpan;
use mago_syntax::cst::AttributeList;
use mago_syntax::cst::PartialArgument;
use mago_syntax::cst::Sequence;
use mago_word::Word;
use mago_word::word;

use crate::flags::attribute::AttributeFlags;
use crate::metadata::attribute::AttributeArgumentMetadata;
use crate::metadata::attribute::AttributeMetadata;
use crate::scanner::Context;
use crate::scanner::inference::infer;

#[inline]
pub fn scan_attribute_lists<'arena, A>(
    attribute_lists: &'arena Sequence<'arena, AttributeList<'arena>>,
    context: &Context<'_, 'arena, A>,
    scope: &NamespaceScope,
    enclosing_class: Option<Word>,
) -> Vec<AttributeMetadata>
where
    A: Arena,
{
    let mut metadata = vec![];

    for attribute_list in attribute_lists {
        for attribute in &attribute_list.attributes {
            let arguments = attribute
                .argument_list
                .iter()
                .flat_map(|arguments| &arguments.arguments)
                .map(|argument| {
                    let (name, name_span, value) = match argument {
                        PartialArgument::Positional(argument) => (None, None, Some(argument.value)),
                        PartialArgument::Named(argument) => {
                            (Some(word(argument.name.value)), Some(argument.name.span), Some(argument.value))
                        }
                        PartialArgument::NamedPlaceholder(argument) => {
                            (Some(word(argument.name.value)), Some(argument.name.span), None)
                        }
                        PartialArgument::Placeholder(_) | PartialArgument::VariadicPlaceholder(_) => (None, None, None),
                    };

                    AttributeArgumentMetadata {
                        name,
                        span: argument.span(),
                        name_span,
                        value_span: value.map(HasSpan::span),
                        value_type: value.and_then(|value| infer(context, scope, value, enclosing_class)),
                    }
                })
                .collect();
            metadata.push(AttributeMetadata {
                name: word(context.resolved_names.get(&attribute.name)),
                span: attribute.span(),
                arguments,
            });
        }
    }

    metadata
}

#[inline]
pub fn get_attribute_flags<'arena, A>(
    class_like_name: Word,
    attribute_lists: &'arena Sequence<'arena, AttributeList<'arena>>,
    context: &Context<'_, 'arena, A>,
    scope: &NamespaceScope,
    classname: Option<Word>,
) -> Option<AttributeFlags>
where
    A: Arena,
{
    if class_like_name.as_bytes().eq_ignore_ascii_case(b"Attribute") {
        return Some(AttributeFlags::TARGET_CLASS);
    }

    for attribute in attribute_lists.iter().flat_map(|list| list.attributes.iter()) {
        let attribute_name = context.resolved_names.get(&attribute.name);
        if !attribute_name.eq_ignore_ascii_case(b"Attribute") {
            continue;
        }

        let Some(first_argument) =
            attribute.argument_list.as_ref().and_then(|argument_list| argument_list.arguments.first())
        else {
            // No target specified means all targets
            return Some(AttributeFlags::TARGET_ALL);
        };

        let Some(value) = first_argument.value() else {
            return None; // Semantically invalid, but we don't want to panic here.
        };

        let inferred_type = infer(context, scope, value, classname);
        let bits = inferred_type.and_then(|i| i.get_single_literal_int_value()).and_then(|value| {
            if !(0..=255).contains(&value) {
                return None;
            }

            Some(value as u8)
        });

        return Some(if let Some(bits) = bits {
            AttributeFlags::from_bits(bits)
        } else {
            // Unable to infer the target, allow all targets + repeatable
            AttributeFlags::all()
        });
    }

    None
}
