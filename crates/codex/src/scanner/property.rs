use mago_atom::Atom;
use mago_atom::atom;
use mago_names::scope::NamespaceScope;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::issue::ScanningIssueKind;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::parameter::FunctionLikeParameterMetadata;
use crate::metadata::property::PropertyMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::VariableIdentifier;
use crate::scanner::Context;
use crate::scanner::docblock::PropertyDocblockComment;
use crate::scanner::inference::infer;
use crate::scanner::ttype::get_type_metadata_from_hint;
use crate::scanner::ttype::get_type_metadata_from_type_string;
use crate::ttype::resolution::TypeResolutionContext;
use crate::visibility::Visibility;

#[inline]
pub fn scan_promoted_property<'arena>(
    parameter: &'arena FunctionLikeParameter<'arena>,
    parameter_metadata: &FunctionLikeParameterMetadata,
    class_metadata: &ClassLikeMetadata,
    context: &mut Context<'_, 'arena>,
) -> PropertyMetadata {
    debug_assert!(parameter.is_promoted_property(), "Parameter is not a promoted property");

    let name = parameter_metadata.get_name();
    let name_span = parameter_metadata.get_name_span();

    let mut flags = MetadataFlags::PROMOTED_PROPERTY;
    if context.file.file_type.is_host() {
        flags |= MetadataFlags::USER_DEFINED;
    } else if context.file.file_type.is_builtin() {
        flags |= MetadataFlags::BUILTIN;
    }

    if parameter_metadata.flags.has_default() {
        flags |= MetadataFlags::HAS_DEFAULT;
    }

    if parameter.modifiers.contains_readonly() {
        flags |= MetadataFlags::READONLY;
    }

    if parameter.modifiers.contains_abstract() {
        flags |= MetadataFlags::ABSTRACT;
    }

    if parameter.modifiers.contains_static() {
        flags |= MetadataFlags::STATIC;
    }

    let default_type_metadata = parameter_metadata.get_default_type().cloned();

    let read_visibility = match parameter.modifiers.get_first_read_visibility() {
        Some(visibility) => Visibility::try_from(visibility).unwrap_or(Visibility::Public),
        None => Visibility::Public,
    };

    let write_visibility = match parameter.modifiers.get_first_write_visibility() {
        Some(visibility) => Visibility::try_from(visibility).unwrap_or(Visibility::Public),
        None => {
            if parameter.modifiers.contains_readonly() {
                Visibility::Protected
            } else {
                read_visibility
            }
        }
    };

    let mut property_metadata = PropertyMetadata::new(*name, flags);

    property_metadata.set_default_type_metadata(default_type_metadata);
    property_metadata.set_name_span(Some(name_span));
    property_metadata.set_span(Some(parameter.span()));
    property_metadata.set_visibility(read_visibility, write_visibility);
    property_metadata.set_is_virtual(parameter.hooks.is_some());
    property_metadata.set_type_declaration_metadata(
        parameter.hint.as_ref().map(|hint| get_type_metadata_from_hint(hint, Some(class_metadata.name), context)),
    );

    if let Some(type_metadata) = parameter_metadata.type_metadata.as_ref()
        && type_metadata.from_docblock
    {
        property_metadata.type_metadata = Some(type_metadata.clone());
    }

    property_metadata
}

#[inline]
pub fn scan_properties<'arena>(
    property: &'arena Property<'arena>,
    class_like_metadata: &mut ClassLikeMetadata,
    classname: Option<Atom>,
    type_context: &TypeResolutionContext,
    context: &mut Context<'_, 'arena>,
    scope: &NamespaceScope,
) -> Vec<PropertyMetadata> {
    let docblock = match PropertyDocblockComment::create(context, property) {
        Ok(docblock) => docblock,
        Err(parse_error) => {
            class_like_metadata.issues.push(
                Issue::error("Failed to parse property docblock comment.")
                    .with_code(ScanningIssueKind::MalformedDocblockComment)
                    .with_annotation(Annotation::primary(parse_error.span()).with_message(parse_error.to_string()))
                    .with_note(parse_error.note())
                    .with_help(parse_error.help()),
            );

            None
        }
    };

    let mut flags = MetadataFlags::empty();
    if context.file.file_type.is_host() {
        flags |= MetadataFlags::USER_DEFINED;
    } else if context.file.file_type.is_builtin() {
        flags |= MetadataFlags::BUILTIN;
    }

    if let Some(docblock) = docblock.as_ref() {
        if docblock.is_internal {
            flags |= MetadataFlags::INTERNAL;
        }

        if docblock.is_deprecated {
            flags |= MetadataFlags::DEPRECATED;
        }

        if docblock.is_readonly {
            flags |= MetadataFlags::READONLY;
        }
    }

    match property {
        Property::Plain(plain_property) => plain_property
            .items
            .iter()
            .map(|item| {
                let (name, name_span, has_default, default_type) = scan_property_item(item, context, scope);

                let mut flags = flags;

                if has_default {
                    flags |= MetadataFlags::HAS_DEFAULT;
                }

                if plain_property.modifiers.contains_readonly() {
                    flags |= MetadataFlags::READONLY;
                }

                if plain_property.modifiers.contains_abstract() {
                    flags |= MetadataFlags::ABSTRACT;
                }

                if plain_property.modifiers.contains_static() {
                    flags |= MetadataFlags::STATIC;
                }

                let read_visibility = match plain_property.modifiers.get_first_read_visibility() {
                    Some(visibility) => Visibility::try_from(visibility).unwrap_or(Visibility::Public),
                    None => Visibility::Public,
                };

                let write_visibility = match plain_property.modifiers.get_first_write_visibility() {
                    Some(visibility) => Visibility::try_from(visibility).unwrap_or(Visibility::Public),
                    None => {
                        if plain_property.modifiers.contains_readonly() {
                            Visibility::Protected
                        } else {
                            read_visibility
                        }
                    }
                };

                let mut metadata = PropertyMetadata::new(name, flags);

                metadata.set_name_span(Some(name_span));
                metadata.set_default_type_metadata(default_type);
                metadata.set_visibility(read_visibility, write_visibility);
                metadata.set_type_declaration_metadata(
                    plain_property
                        .hint
                        .as_ref()
                        .map(|hint| get_type_metadata_from_hint(hint, Some(class_like_metadata.name), context)),
                );

                if let Some(docblock) = docblock.as_ref() {
                    if let Some(type_string) = &docblock.type_string {
                        match get_type_metadata_from_type_string(type_string, classname, type_context, scope) {
                            Ok(property_type_metadata) => {
                                metadata.set_type_metadata(Some(property_type_metadata));
                            }
                            Err(typing_error) => class_like_metadata.issues.push(
                                Issue::error("Could not resolve the type for the @var tag.")
                                    .with_code(ScanningIssueKind::InvalidVarTag)
                                    .with_annotation(
                                        Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                                    )
                                    .with_note(typing_error.note())
                                    .with_help(typing_error.help()),
                            ),
                        }
                    }

                    metadata
                } else {
                    metadata
                }
            })
            .collect(),
        Property::Hooked(hooked_property) => {
            let (name, name_span, has_default, default_type) =
                scan_property_item(&hooked_property.item, context, scope);

            let visibility = match hooked_property.modifiers.get_first_visibility() {
                Some(visibility) => Visibility::try_from(visibility).unwrap_or(Visibility::Public),
                None => Visibility::Public,
            };

            if has_default {
                flags |= MetadataFlags::HAS_DEFAULT;
            }

            if hooked_property.modifiers.contains_abstract() {
                flags |= MetadataFlags::ABSTRACT;
            }

            let mut metadata = PropertyMetadata::new(name, flags);

            metadata.set_name_span(Some(name_span));
            metadata.set_default_type_metadata(default_type);
            metadata.set_span(Some(hooked_property.span()));
            metadata.set_visibility(visibility, visibility);
            metadata.set_type_declaration_metadata(
                hooked_property
                    .hint
                    .as_ref()
                    .map(|hint| get_type_metadata_from_hint(hint, Some(class_like_metadata.name), context)),
            );

            if let Some(docblock) = docblock.as_ref()
                && let Some(type_string) = &docblock.type_string
            {
                match get_type_metadata_from_type_string(type_string, classname, type_context, scope) {
                    Ok(property_type) => {
                        metadata.set_type_metadata(Some(property_type));
                    }
                    Err(typing_error) => {
                        class_like_metadata.issues.push(
                            Issue::error("Could not resolve the type for the @var tag.")
                                .with_code(ScanningIssueKind::InvalidVarTag)
                                .with_annotation(
                                    Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                                )
                                .with_note(typing_error.note())
                                .with_help(typing_error.help()),
                        );
                    }
                }
            }

            vec![metadata]
        }
    }
}

#[inline]
pub fn scan_property_item<'arena>(
    property_item: &'arena PropertyItem<'arena>,
    context: &mut Context<'_, 'arena>,
    scope: &NamespaceScope,
) -> (VariableIdentifier, Span, bool, Option<TypeMetadata>) {
    match property_item {
        PropertyItem::Abstract(property_abstract_item) => {
            let name = VariableIdentifier(atom(property_abstract_item.variable.name));
            let name_span = property_abstract_item.variable.span;
            let has_default = false;
            let default_type = None;

            (name, name_span, has_default, default_type)
        }
        PropertyItem::Concrete(property_concrete_item) => {
            let name = VariableIdentifier(atom(property_concrete_item.variable.name));
            let name_span = property_concrete_item.variable.span;
            let has_default = true;
            let default_type = infer(context, scope, &property_concrete_item.value).map(|u| {
                let mut type_metadata = TypeMetadata::new(u, property_concrete_item.value.span());
                type_metadata.inferred = true;
                type_metadata
            });

            (name, name_span, has_default, default_type)
        }
    }
}
