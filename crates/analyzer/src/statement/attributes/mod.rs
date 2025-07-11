use ahash::HashMap;

use mago_codex::flags::attribute::AttributeFlags;
use mago_codex::get_class_like;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::issue::TypingIssueKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum AttributeTarget {
    ClassLike,
    Method,
    Property,
    Parameter,
    PromotedProperty,
    ClassLikeConstant,
    Function,
    Constant,
}

impl AttributeTarget {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ClassLike => "a class, interface, enum, or trait",
            Self::Method => "a method",
            Self::Property => "a property",
            Self::Parameter => "a parameter",
            Self::PromotedProperty => "a promoted property",
            Self::ClassLikeConstant => "a class constant",
            Self::Function => "a function",
            Self::Constant => "a constant",
        }
    }
}

pub fn analyze_attributes(
    context: &mut Context<'_>,
    _block_context: &mut BlockContext<'_>,
    _artifacts: &mut AnalysisArtifacts,
    attribute_lists: &[AttributeList],
    target: AttributeTarget,
) -> Result<(), AnalysisError> {
    let attributes_names = attribute_lists.iter().flat_map(|list| list.attributes.iter()).collect::<Vec<_>>();

    let mut used_attributes = HashMap::default();
    for attribute in attributes_names {
        let attribute_name = context.resolved_names.get(&attribute.name);
        let attribute_name_str = context.interner.lookup(attribute_name);

        let Some(metadata) = get_class_like(context.codebase, context.interner, attribute_name) else {
            context.buffer.report(
                TypingIssueKind::NonExistentAttributeClass,
                Issue::error(format!("Attribute class `{attribute_name_str}` not found or could not be autoloaded."))
                .with_annotation(
                    Annotation::primary(attribute.span()).with_message(format!("Unknown attribute class `{attribute_name_str}`")),
                )
                .with_note("Attributes must be classes that are defined, correctly namespaced, and autoloadable. Ensure the class exists and is accessible.")
                .with_help("Verify the attribute class name, its namespace, and your autoloader configuration. Make sure the class is defined."),
            );

            continue;
        };

        let class_like_kind_str = metadata.kind.as_str();

        if !metadata.kind.is_class() {
            context.buffer.report(
                TypingIssueKind::NonClassUsedAsAttribute,
                Issue::error(format!(
                    "The {class_like_kind_str} `{attribute_name_str}` cannot be used as an attribute.",
                ))
                .with_annotation(
                    Annotation::primary(attribute.span())
                        .with_message(format!(
                            "`{attribute_name_str}` is a{} {class_like_kind_str} and not a class",
                            if metadata.kind.is_interface() || metadata.kind.is_enum() { "n" } else { "" }
                        )),
                )
                .with_annotation(
                    Annotation::secondary(metadata.get_name_span().unwrap_or_else(|| metadata.get_span()))
                        .with_message(format!(
                            "`{attribute_name_str}` defined as a{} {class_like_kind_str} here",
                            if metadata.kind.is_interface() || metadata.kind.is_enum() { "n" } else { "" }
                        )),
                )
                .with_note("Only classes can be declared as attributes.")
                .with_note("Interfaces, enums, and traits are not valid attribute types.")
                .with_help(format!("Ensure you are using a class intended to be an attribute. Replace `{attribute_name_str}` with a valid attribute class.")),
            );

            continue;
        }

        if metadata.is_abstract() {
            context.buffer.report(
                TypingIssueKind::AbstractClassUsedAsAttribute,
                Issue::error(format!("The abstract class `{attribute_name_str}` cannot be used as an attribute.",))
                    .with_annotation(Annotation::primary(attribute.span()).with_message(format!(
                        "`{attribute_name_str}` is an abstract class and cannot be instantiated as an attribute"
                    )))
                    .with_annotation(
                        Annotation::secondary(metadata.get_name_span().unwrap_or_else(|| metadata.get_span()))
                            .with_message(format!("`{attribute_name_str}` defined here as an abstract class")),
                    )
                    .with_note("Attributes must be concrete classes that can be instantiated.")
                    .with_help(format!("Use a concrete class instead of `{attribute_name_str}` for attributes.")),
            );

            continue;
        }

        let Some(attribute_flags) = metadata.get_attribute_flags() else {
            context.buffer.report(
                TypingIssueKind::ClassNotMarkedAsAttribute,
                Issue::error(format!(
                    "Class `{attribute_name_str}` is used as an attribute but is not declared with `#[Attribute]`.",
                ))
                .with_annotation(
                    Annotation::primary(attribute.span()).with_message(format!("`{attribute_name_str}` used as an attribute here")),
                )
                .with_annotation(
                    Annotation::secondary(metadata.get_name_span().unwrap_or_else(|| metadata.get_span()))
                        .with_message(format!("Class `{attribute_name_str}` defined here needs an `#[Attribute]` declaration")),
                )
                .with_note("To be used as a PHP attribute, a class must itself be decorated with the `#[\\Attribute]` system attribute.")
                .with_help(format!("Add `#[\\Attribute]` to the definition of class `{attribute_name_str}` to declare it as an attribute, or use a different class that is a valid attribute.")),
            );

            continue;
        };

        if let Some(first_usage_span) = used_attributes.get(attribute_name)
            && !attribute_flags.is_repeatable()
        {
            context.buffer.report(
                TypingIssueKind::AttributeNotRepeatable,
                Issue::error(format!("Attribute `{attribute_name_str}` is not declared as repeatable and has already been used."))
                .with_annotation(
                    Annotation::primary(attribute.span())
                        .with_message(format!("Duplicate use of non-repeatable attribute `{attribute_name_str}`")),
                )
                .with_annotation(
                    Annotation::secondary(*first_usage_span)
                        .with_message(format!("Attribute `{attribute_name_str}` was first used here")),
                )
                .with_note(format!(
                    "The attribute `{attribute_name_str}` is not declared with `Attribute::IS_REPEATABLE` in its `#[Attribute]` flags. Non-repeatable attributes can only be applied once to a given target (e.g., a class, method, property).",
                ))
                .with_help(format!(
                    "Remove this duplicate `{attribute_name_str}` attribute, or if multiple instances are intended and valid, modify the attribute class `{attribute_name_str}` to include `Attribute::IS_REPEATABLE` in its `#[Attribute]` declaration (e.g., `#[Attribute(Attribute::TARGET_ALL | Attribute::IS_REPEATABLE)]`).",
                )),
            );

            continue;
        }

        used_attributes.insert(attribute_name, attribute.span());

        if let Some(flags) = metadata.attribute_flags {
            let is_valid_target = match target {
                AttributeTarget::ClassLike => flags.targets_class(),
                AttributeTarget::Method => flags.targets_method(),
                AttributeTarget::Property => flags.targets_property(),
                AttributeTarget::Parameter => flags.targets_parameter(),
                AttributeTarget::PromotedProperty => flags.targets_property() || flags.targets_parameter(),
                AttributeTarget::ClassLikeConstant => flags.targets_class_constant(),
                AttributeTarget::Function => flags.targets_function(),
                AttributeTarget::Constant => flags.targets_constant(),
            };

            if !is_valid_target {
                report_invalid_target(context, metadata, attribute, target, flags);
            }
        }
    }

    Ok(())
}

fn report_invalid_target(
    context: &mut Context<'_>,
    metadata: &ClassLikeMetadata,
    attribute: &Attribute,
    target: AttributeTarget,
    flags: AttributeFlags,
) {
    let attribute_name_str = context.interner.lookup(&metadata.original_name);
    let short_attribute_name_str = attribute_name_str.split("\\").last().unwrap_or(attribute_name_str);
    let allowed_targets = flags.get_target_names().join(", ");

    context.buffer.report(
        TypingIssueKind::InvalidAttributeTarget,
        Issue::error(format!("Attribute `{attribute_name_str}` cannot be used on {}.", target.as_str()))
            .with_annotation(Annotation::primary(attribute.span()).with_message("This attribute is not allowed here"))
            .with_annotation(
                Annotation::secondary(metadata.get_name_span().unwrap_or_else(|| metadata.get_span()))
                    .with_message(format!("`{attribute_name_str}` defined here")),
            )
            .with_note(format!(
                "The definition of `{attribute_name_str}` restricts its use to the following targets: {allowed_targets}."
            ))
            .with_help(format!(
                "Remove the `#[{short_attribute_name_str}]` attribute from this location, or update the `#[Attribute]` declaration on the `{attribute_name_str}` class to include `{}` as a valid target.",
                target.as_str()
            ))
    );
}
