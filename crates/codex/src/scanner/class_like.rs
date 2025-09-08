use mago_atom::Atom;
use mago_atom::ascii_lowercase_atom;
use mago_atom::atom;
use mago_names::scope::NamespaceScope;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::consts::MAX_ENUM_CASES_FOR_ANALYSIS;
use crate::get_anonymous_class_name;
use crate::issue::ScanningIssueKind;
use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::property::PropertyMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::GenericParent;
use crate::misc::VariableIdentifier;
use crate::scanner::Context;
use crate::scanner::TemplateConstraintList;
use crate::scanner::attribute::get_attribute_flags;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::class_like_constant::scan_class_like_constants;
use crate::scanner::docblock::ClassLikeDocblockComment;
use crate::scanner::docblock::TraitUseDocblockComment;
use crate::scanner::enum_case::scan_enum_case;
use crate::scanner::property::scan_properties;
use crate::symbol::SymbolKind;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::builder;
use crate::ttype::get_mixed;
use crate::ttype::get_string;
use crate::ttype::resolution::TypeResolutionContext;
use crate::ttype::template::variance::Variance;
use crate::ttype::union::TUnion;
use crate::visibility::Visibility;

#[inline]
pub fn register_anonymous_class<'ctx, 'arena>(
    codebase: &mut CodebaseMetadata,
    class: &'arena AnonymousClass<'arena>,
    context: &mut Context<'ctx, 'arena>,
    scope: &mut NamespaceScope,
) -> Option<(Atom, TemplateConstraintList)> {
    let span = class.span();
    let name = get_anonymous_class_name(span);

    let class_like_metadata = scan_class_like(
        codebase,
        name,
        SymbolKind::Class,
        None,
        span,
        &class.attribute_lists,
        Some(&class.modifiers),
        &class.members,
        class.extends.as_ref(),
        class.implements.as_ref(),
        None,
        context,
        scope,
    )?;

    let template_resolution_context = class_like_metadata
        .template_types
        .iter()
        .map(|(name, definition)| (*name, definition.clone()))
        .collect::<TemplateConstraintList>();

    codebase.class_likes.insert(name, class_like_metadata);

    Some((name, template_resolution_context))
}

#[inline]
pub fn register_class<'ctx, 'arena>(
    codebase: &mut CodebaseMetadata,
    class: &'arena Class<'arena>,
    context: &mut Context<'ctx, 'arena>,
    scope: &mut NamespaceScope,
) -> Option<(Atom, TemplateConstraintList)> {
    let class_like_metadata = scan_class_like(
        codebase,
        atom(context.resolved_names.get(&class.name)),
        SymbolKind::Class,
        Some(class.name.span),
        class.span(),
        &class.attribute_lists,
        Some(&class.modifiers),
        &class.members,
        class.extends.as_ref(),
        class.implements.as_ref(),
        None,
        context,
        scope,
    )?;

    let template_resolution_context = class_like_metadata
        .template_types
        .iter()
        .map(|(name, definition)| (*name, definition.clone()))
        .collect::<TemplateConstraintList>();

    let name = class_like_metadata.name;

    codebase.class_likes.insert(name, class_like_metadata);

    Some((name, template_resolution_context))
}

#[inline]
pub fn register_interface<'ctx, 'arena>(
    codebase: &mut CodebaseMetadata,
    interface: &'arena Interface<'arena>,
    context: &mut Context<'ctx, 'arena>,
    scope: &mut NamespaceScope,
) -> Option<(Atom, TemplateConstraintList)> {
    let class_like_metadata = scan_class_like(
        codebase,
        atom(context.resolved_names.get(&interface.name)),
        SymbolKind::Interface,
        Some(interface.name.span),
        interface.span(),
        &interface.attribute_lists,
        None,
        &interface.members,
        interface.extends.as_ref(),
        None,
        None,
        context,
        scope,
    )?;

    let template_resolution_context = class_like_metadata
        .template_types
        .iter()
        .map(|(name, definition)| (*name, definition.clone()))
        .collect::<TemplateConstraintList>();

    let name = class_like_metadata.name;

    codebase.class_likes.insert(name, class_like_metadata);

    Some((name, template_resolution_context))
}

#[inline]
pub fn register_trait<'ctx, 'arena>(
    codebase: &mut CodebaseMetadata,
    r#trait: &'arena Trait<'arena>,
    context: &mut Context<'ctx, 'arena>,
    scope: &mut NamespaceScope,
) -> Option<(Atom, TemplateConstraintList)> {
    let class_like_metadata = scan_class_like(
        codebase,
        atom(context.resolved_names.get(&r#trait.name)),
        SymbolKind::Trait,
        Some(r#trait.name.span),
        r#trait.span(),
        &r#trait.attribute_lists,
        None,
        &r#trait.members,
        None,
        None,
        None,
        context,
        scope,
    )?;

    let template_resolution_context = class_like_metadata
        .template_types
        .iter()
        .map(|(name, definition)| (*name, definition.clone()))
        .collect::<TemplateConstraintList>();

    let name = class_like_metadata.name;

    codebase.class_likes.insert(name, class_like_metadata);

    Some((name, template_resolution_context))
}

#[inline]
pub fn register_enum<'ctx, 'arena>(
    codebase: &mut CodebaseMetadata,
    r#enum: &'arena Enum<'arena>,
    context: &mut Context<'ctx, 'arena>,
    scope: &mut NamespaceScope,
) -> Option<(Atom, TemplateConstraintList)> {
    let class_like_metadata = scan_class_like(
        codebase,
        atom(context.resolved_names.get(&r#enum.name)),
        SymbolKind::Enum,
        Some(r#enum.name.span),
        r#enum.span(),
        &r#enum.attribute_lists,
        None,
        &r#enum.members,
        None,
        r#enum.implements.as_ref(),
        r#enum.backing_type_hint.as_ref(),
        context,
        scope,
    )?;

    let template_resolution_context = class_like_metadata
        .template_types
        .iter()
        .map(|(name, definition)| (*name, definition.clone()))
        .collect::<TemplateConstraintList>();

    let name = class_like_metadata.name;

    codebase.class_likes.insert(name, class_like_metadata);

    Some((name, template_resolution_context))
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn scan_class_like<'ctx, 'arena>(
    codebase: &mut CodebaseMetadata,
    name: Atom,
    kind: SymbolKind,
    name_span: Option<Span>,
    span: Span,
    attribute_lists: &'arena Sequence<'arena, AttributeList<'arena>>,
    modifiers: Option<&'arena Sequence<Modifier<'arena>>>,
    members: &'arena Sequence<ClassLikeMember<'arena>>,
    extends: Option<&'arena Extends<'arena>>,
    implements: Option<&'arena Implements<'arena>>,
    enum_type: Option<&'arena EnumBackingTypeHint<'arena>>,
    context: &mut Context<'ctx, 'arena>,
    scope: &mut NamespaceScope,
) -> Option<ClassLikeMetadata> {
    let original_name = name;
    let name = ascii_lowercase_atom(&original_name);

    if codebase.class_likes.contains_key(&name) {
        return None;
    }

    let mut flags = MetadataFlags::empty();
    if context.file.file_type.is_host() {
        flags |= MetadataFlags::USER_DEFINED;
    } else if context.file.file_type.is_builtin() {
        flags |= MetadataFlags::BUILTIN;
    }

    let mut class_like_metadata = ClassLikeMetadata::new(name, original_name, span, name_span, flags);

    class_like_metadata.attributes = scan_attribute_lists(attribute_lists, context);
    class_like_metadata.enum_type = match enum_type {
        Some(EnumBackingTypeHint { hint: Hint::String(_), .. }) => Some(TAtomic::Scalar(TScalar::string())),
        Some(EnumBackingTypeHint { hint: Hint::Integer(_), .. }) => Some(TAtomic::Scalar(TScalar::int())),
        _ => None,
    };

    if kind.is_class() {
        class_like_metadata.attribute_flags = get_attribute_flags(name, attribute_lists, context);
    }

    class_like_metadata.kind = kind;

    match kind {
        SymbolKind::Class => {
            if modifiers.is_some_and(|m| m.contains_final()) {
                class_like_metadata.flags |= MetadataFlags::FINAL;
            }

            if modifiers.is_some_and(|m| m.contains_abstract()) {
                class_like_metadata.flags |= MetadataFlags::ABSTRACT;
            }

            if modifiers.is_some_and(|m| m.contains_readonly()) {
                class_like_metadata.flags |= MetadataFlags::READONLY;
            }

            codebase.symbols.add_class_name(name);

            if let Some(extended_class) = extends.and_then(|e| e.types.first()) {
                let parent_name = context.resolved_names.get(extended_class);
                let parent_name = ascii_lowercase_atom(parent_name);

                class_like_metadata.direct_parent_class = Some(parent_name);
                class_like_metadata.all_parent_classes.insert(parent_name);
            }
        }
        SymbolKind::Enum => {
            class_like_metadata.flags |= MetadataFlags::FINAL;

            if enum_type.is_some() {
                let backed_enum_interface = atom("backedenum");
                let from_method = atom("from");
                let try_from_method = atom("tryfrom");

                class_like_metadata.all_parent_interfaces.insert(backed_enum_interface);
                class_like_metadata.direct_parent_interfaces.insert(backed_enum_interface);

                class_like_metadata.appearing_method_ids.insert(from_method, backed_enum_interface);
                class_like_metadata.declaring_method_ids.insert(from_method, backed_enum_interface);
                class_like_metadata.appearing_method_ids.insert(try_from_method, backed_enum_interface);
                class_like_metadata.declaring_method_ids.insert(try_from_method, backed_enum_interface);
            }

            let unit_enum_interface = atom("unitenum");
            let cases_method = atom("cases");

            class_like_metadata.all_parent_interfaces.insert(unit_enum_interface);
            class_like_metadata.direct_parent_interfaces.insert(unit_enum_interface);

            class_like_metadata.appearing_method_ids.insert(cases_method, unit_enum_interface);
            class_like_metadata.declaring_method_ids.insert(cases_method, unit_enum_interface);

            codebase.symbols.add_enum_name(name);
        }
        SymbolKind::Trait => {
            if class_like_metadata.attributes.iter().any(|attr| attr.name.eq_ignore_ascii_case("Deprecated")) {
                class_like_metadata.flags |= MetadataFlags::DEPRECATED;
            }

            codebase.symbols.add_trait_name(name);
        }
        SymbolKind::Interface => {
            class_like_metadata.flags |= MetadataFlags::ABSTRACT;

            codebase.symbols.add_interface_name(name);

            if let Some(extends) = extends {
                for extended_interface in extends.types.iter() {
                    let parent_name = context.resolved_names.get(extended_interface);
                    let parent_name = ascii_lowercase_atom(parent_name);

                    class_like_metadata.add_direct_parent_interface(parent_name);
                }
            }
        }
    };

    if (class_like_metadata.kind.is_class() || class_like_metadata.kind.is_enum())
        && let Some(implemented_interfaces) = implements
    {
        for interface_name in implemented_interfaces.types.iter() {
            let interface_name = context.resolved_names.get(interface_name);
            let interface_name = ascii_lowercase_atom(interface_name);

            class_like_metadata.add_direct_parent_interface(interface_name);
        }
    }

    let mut type_context = TypeResolutionContext::new();
    let docblock = match ClassLikeDocblockComment::create(context, span, scope) {
        Ok(docblock) => docblock,
        Err(parse_error) => {
            class_like_metadata.issues.push(
                Issue::error("Failed to parse class-like docblock comment.")
                    .with_code(ScanningIssueKind::MalformedDocblockComment)
                    .with_annotation(Annotation::primary(parse_error.span()).with_message(parse_error.to_string()))
                    .with_note(parse_error.note())
                    .with_help(parse_error.help()),
            );

            None
        }
    };

    if let Some(docblock) = docblock {
        if class_like_metadata.kind.is_interface() && docblock.is_enum_interface {
            class_like_metadata.flags |= MetadataFlags::ENUM_INTERFACE;
        }

        if docblock.is_final {
            class_like_metadata.flags |= MetadataFlags::FINAL;
        }

        if docblock.is_deprecated {
            class_like_metadata.flags |= MetadataFlags::DEPRECATED;
        }

        if docblock.is_internal {
            class_like_metadata.flags |= MetadataFlags::INTERNAL;
        }

        if docblock.has_consistent_constructor {
            class_like_metadata.flags |= MetadataFlags::CONSISTENT_CONSTRUCTOR;
        }

        if docblock.has_consistent_templates {
            class_like_metadata.flags |= MetadataFlags::CONSISTENT_TEMPLATES;
        }

        class_like_metadata.has_sealed_methods = docblock.has_sealed_methods;
        class_like_metadata.has_sealed_properties = docblock.has_sealed_properties;

        for (i, template) in docblock.templates.iter().enumerate() {
            let template_name = atom(&template.name);
            let template_as_type = if let Some(type_string) = &template.type_string {
                match builder::get_type_from_string(
                    &type_string.value,
                    type_string.span,
                    scope,
                    &type_context,
                    Some(name),
                ) {
                    Ok(tunion) => tunion,
                    Err(typing_error) => {
                        class_like_metadata.issues.push(
                            Issue::error("Could not resolve the constraint type for the `@template` tag.")
                                .with_code(ScanningIssueKind::InvalidTemplateTag)
                                .with_annotation(
                                    Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                                )
                                .with_note(typing_error.note())
                                .with_help(typing_error.help()),
                        );

                        continue;
                    }
                }
            } else {
                get_mixed()
            };

            let definition = vec![(GenericParent::ClassLike(name), template_as_type)];

            class_like_metadata.add_template_type((template_name, definition.clone()));
            type_context = type_context.with_template_definition(template_name, definition);

            let variance = if template.covariant {
                Variance::Covariant
            } else if template.contravariant {
                Variance::Contravariant
            } else {
                Variance::Invariant
            };

            if variance.is_readonly() {
                class_like_metadata.template_readonly.insert(template_name);
            }

            class_like_metadata.add_template_variance_parameter(i, variance);
        }

        for extended_type in docblock.template_extends {
            let extended_union = match builder::get_type_from_string(
                &extended_type.value,
                extended_type.span,
                scope,
                &type_context,
                Some(name),
            ) {
                Ok(tunion) => tunion,
                Err(typing_error) => {
                    class_like_metadata.issues.push(
                        Issue::error("Could not resolve the generic type in the `@extends` tag.")
                            .with_code(ScanningIssueKind::InvalidExtendsTag)
                            .with_annotation(
                                Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                            )
                            .with_note(typing_error.note())
                            .with_help(typing_error.help()),
                    );

                    continue;
                }
            };

            if !extended_union.is_single() {
                class_like_metadata.issues.push(
                    Issue::error("The `@extends` tag must specify a single parent class.")
                        .with_code(ScanningIssueKind::InvalidExtendsTag)
                        .with_annotation(
                            Annotation::primary(extended_type.span).with_message("Union types are not allowed here."),
                        )
                        .with_note("The `@extends` tag provides concrete types for generics from a direct parent type.")
                        .with_help("Provide a single parent type, e.g., `@extends Box<string>`."),
                );

                continue;
            }

            let (parent_name, parent_parameters) = match extended_union.get_single_owned() {
                TAtomic::Reference(TReference::Symbol { name, parameters, intersection_types: None }) => {
                    (name, parameters)
                }
                _ => {
                    class_like_metadata.issues.push(
                        Issue::error("The `@extends` tag expects a generic class type.")
                            .with_code(ScanningIssueKind::InvalidExtendsTag)
                            .with_annotation(
                                Annotation::primary(extended_type.span)
                                    .with_message("This must be a class name, not a primitive or other complex type."),
                            )
                            .with_note(
                                "The `@extends` tag provides concrete types for type parameters from a direct parent class.",
                            )
                            .with_help("For example: `@extends Box<string>`."),
                    );

                    continue;
                }
            };

            let lowercase_parent_name = ascii_lowercase_atom(&parent_name);

            let has_parent = if class_like_metadata.kind.is_interface() {
                class_like_metadata.all_parent_interfaces.contains(&lowercase_parent_name)
            } else {
                class_like_metadata.all_parent_classes.contains(&lowercase_parent_name)
            };

            if !has_parent {
                class_like_metadata.issues.push(
                    Issue::error("`@extends` tag must refer to a direct parent class or interface.")
                        .with_code(ScanningIssueKind::InvalidExtendsTag)
                        .with_annotation(Annotation::primary(extended_type.span).with_message(format!(
                            "The class `{parent_name}` is not a direct parent."
                        )))
                        .with_note("The `@extends` tag is used to provide type information for the class or interface that is directly extended.")
                        .with_help(format!("Ensure this type's definition includes `extends {parent_name}`.")),
                );

                continue;
            }

            if let Some(extended_parent_parameters) = parent_parameters {
                class_like_metadata
                    .template_type_extends_count
                    .insert(lowercase_parent_name, extended_parent_parameters.len());
                class_like_metadata.add_template_extended_offset(lowercase_parent_name, extended_parent_parameters);
            }
        }

        for implemented_type in docblock.template_implements {
            let implemented_union = match builder::get_type_from_string(
                &implemented_type.value,
                implemented_type.span,
                scope,
                &type_context,
                Some(name),
            ) {
                Ok(tunion) => tunion,
                Err(typing_error) => {
                    class_like_metadata.issues.push(
                        Issue::error("Could not resolve the interface name in the `@implements` tag.")
                            .with_code(ScanningIssueKind::InvalidImplementsTag)
                            .with_annotation(
                                Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                            )
                            .with_note(typing_error.note())
                            .with_help(typing_error.help()),
                    );

                    continue;
                }
            };

            if !implemented_union.is_single() {
                class_like_metadata.issues.push(
                    Issue::error("The `@implements` tag expects a single interface type.")
                        .with_code(ScanningIssueKind::InvalidImplementsTag)
                        .with_annotation(
                            Annotation::primary(implemented_type.span).with_message("Union types are not supported here."),
                        )
                        .with_note("The `@implements` tag provides concrete types for generics from a direct parent interface.")
                        .with_help("Provide a single parent interface, e.g., `@implements Serializable<string>`."),
                );

                continue;
            }

            let (parent_name, parent_parameters) = match implemented_union.get_single_owned() {
                TAtomic::Reference(TReference::Symbol { name, parameters, intersection_types: None }) => {
                    (name, parameters)
                }
                atomic => {
                    let atomic_str = atomic.get_id();

                    class_like_metadata.issues.push(
                        Issue::error("The `@implements` tag expects a single interface type.")
                            .with_code(ScanningIssueKind::InvalidImplementsTag)
                            .with_annotation(
                                Annotation::primary(implemented_type.span)
                                    .with_message(format!("This must be an interface, not `{atomic_str}`.")),
                            )
                            .with_note("The `@implements` tag provides concrete types for type parameters from a direct parent interface.")
                            .with_help("Provide the single, interface name that this class implements."),
                    );

                    continue;
                }
            };

            let lowercase_parent_name = ascii_lowercase_atom(&parent_name);

            if !class_like_metadata.all_parent_interfaces.contains(&lowercase_parent_name) {
                class_like_metadata.issues.push(
                    Issue::error("The `@implements` tag must refer to a direct parent interface.")
                        .with_code(ScanningIssueKind::InvalidImplementsTag)
                        .with_annotation(Annotation::primary(implemented_type.span).with_message(format!(
                            "The interface `{parent_name}` is not a direct parent."
                        )))
                        .with_note("The `@implements` tag is used to provide type information for the interface that is directly implemented.")
                        .with_help(format!("Ensure this type's definition includes `implements {parent_name}`.")),
                );

                continue;
            }

            if let Some(impl_parent_parameters) = parent_parameters {
                class_like_metadata
                    .template_type_implements_count
                    .insert(lowercase_parent_name, impl_parent_parameters.len());
                class_like_metadata.add_template_extended_offset(lowercase_parent_name, impl_parent_parameters);
            }
        }

        for require_extend in docblock.require_extends {
            let required_union = match builder::get_type_from_string(
                &require_extend.value,
                require_extend.span,
                scope,
                &type_context,
                Some(name),
            ) {
                Ok(tunion) => tunion,
                Err(typing_error) => {
                    class_like_metadata.issues.push(
                        Issue::error("Could not resolve the class name in the `@require-extends` tag.")
                            .with_code(ScanningIssueKind::InvalidRequireExtendsTag)
                            .with_annotation(
                                Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                            )
                            .with_note(typing_error.note())
                            .with_help(typing_error.help()),
                    );

                    continue;
                }
            };

            if !required_union.is_single() {
                class_like_metadata.issues.push(
                    Issue::error("The `@require-extends` tag expects a single class name.")
                        .with_code(ScanningIssueKind::InvalidRequireExtendsTag)
                        .with_annotation(
                            Annotation::primary(require_extend.span)
                                .with_message("Union types are not supported here."),
                        )
                        .with_note("The `@require-extends` tag forces any type that inherits from this one to also extend a specific base class.")
                        .with_help("A class can only extend one other class. Provide a single parent class name."),
                );

                continue;
            }

            let (required_name, required_params) = match required_union.get_single_owned() {
                TAtomic::Reference(TReference::Symbol { name, parameters, intersection_types }) => {
                    if intersection_types.is_some() {
                        class_like_metadata.issues.push(
                            Issue::error("The `@require-extends` tag expects a single class name.")
                                .with_code(ScanningIssueKind::InvalidRequireExtendsTag)
                                .with_annotation(
                                    Annotation::primary(require_extend.span)
                                        .with_message("Intersection types are not supported here."),
                                )
                                .with_note("The `@require-extends` tag forces any type that inherits from this one to also extend a specific base class.")
                                .with_help("A class can only extend one other class. Provide a single parent class name."),
                        );

                        continue;
                    }

                    (name, parameters)
                }
                _ => {
                    class_like_metadata.issues.push(
                        Issue::error("The `@require-extends` tag expects a single class name.")
                            .with_code(ScanningIssueKind::InvalidRequireExtendsTag)
                            .with_annotation(
                                Annotation::primary(require_extend.span)
                                    .with_message("This must be a class name, not a primitive or other complex type.")
                            )
                            .with_note("The `@require-extends` tag forces any type that inherits from this one to also extend a specific base class.")
                            .with_help("Provide the single, class name that all inheriting classes must extend."),
                    );

                    continue;
                }
            };

            class_like_metadata.require_extends.insert(ascii_lowercase_atom(&required_name));
            if let Some(required_params) = required_params {
                class_like_metadata.add_template_extended_offset(required_name, required_params);
            }
        }

        for require_implements in docblock.require_implements {
            let required_union = match builder::get_type_from_string(
                &require_implements.value,
                require_implements.span,
                scope,
                &type_context,
                Some(name),
            ) {
                Ok(tunion) => tunion,
                Err(typing_error) => {
                    class_like_metadata.issues.push(
                        Issue::error("Could not resolve the interface name in the `@require-implements` tag.")
                            .with_code(ScanningIssueKind::InvalidRequireImplementsTag)
                            .with_annotation(
                                Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                            )
                            .with_note(typing_error.note())
                            .with_help(typing_error.help()),
                    );

                    continue;
                }
            };

            if !required_union.is_single() {
                class_like_metadata.issues.push(
                    Issue::error("The `@require-implements` tag expects a single interface name.")
                        .with_code(ScanningIssueKind::InvalidRequireImplementsTag)
                        .with_annotation(
                            Annotation::primary(require_implements.span)
                                .with_message("Union types are not supported here."),
                        )
                        .with_note("The `@require-implements` tag forces any type that inherits from this one to also implement a specific interface.")
                        .with_help("To require that inheriting types implement multiple interfaces, use a separate `@require-implements` tag for each one."),
                );

                continue;
            }

            let (required_name, required_parameters) = match required_union.get_single_owned() {
                TAtomic::Reference(TReference::Symbol { name, parameters, intersection_types }) => {
                    if intersection_types.is_some() {
                        class_like_metadata.issues.push(
                            Issue::error("The `@require-implements` tag expects a single interface name.")
                                .with_code(ScanningIssueKind::InvalidRequireImplementsTag)
                                .with_annotation(
                                    Annotation::primary(require_implements.span)
                                        .with_message("Intersection types are not supported here."),
                                )
                                .with_note("The `@require-implements` tag forces any type that inherits from this one to also implement a specific interface.")
                                .with_help("To require that inheriting types implement multiple interfaces, use a separate `@require-implements` tag for each one."),
                        );

                        continue;
                    }

                    (name, parameters)
                }
                _ => {
                    class_like_metadata.issues.push(
                        Issue::error("The `@require-implements` tag expects a single interface name.")
                            .with_code(ScanningIssueKind::InvalidRequireImplementsTag)
                            .with_annotation(
                                Annotation::primary(require_implements.span)
                                    .with_message("This must be an interface, not a primitive or other complex type."),
                            )
                            .with_note("The `@require-implements` tag forces any type that inherits from this one to also implement a specific interface.")
                            .with_help("Provide the single, interface name that all inheriting classes must implement."),
                    );

                    continue;
                }
            };

            class_like_metadata.require_implements.insert(ascii_lowercase_atom(&required_name));
            if let Some(required_parameters) = required_parameters {
                class_like_metadata.add_template_extended_offset(required_name, required_parameters);
            }
        }

        if let Some(inheritors) = docblock.inheritors {
            match builder::get_type_from_string(&inheritors.value, inheritors.span, scope, &type_context, Some(name)) {
                Ok(inheritors_union) => {
                    for inheritor in inheritors_union.types.as_ref() {
                        match inheritor {
                            TAtomic::Reference(TReference::Symbol { name, intersection_types: None, .. }) => {
                                class_like_metadata
                                    .permitted_inheritors
                                    .get_or_insert_default()
                                    .insert(ascii_lowercase_atom(name));
                            }
                            _ => {
                                class_like_metadata.issues.push(
                                    Issue::error("The `@inheritors` tag only accepts class, interface, or enum names.")
                                        .with_code(ScanningIssueKind::InvalidInheritorsTag)
                                        .with_annotation(
                                            Annotation::primary(inheritors.span)
                                                .with_message("This type is not a simple class-like name."),
                                        ),
                                );
                            }
                        }
                    }
                }
                Err(typing_error) => {
                    class_like_metadata.issues.push(
                        Issue::error("Could not resolve the type in the `@inheritors` tag.")
                            .with_code(ScanningIssueKind::InvalidInheritorsTag)
                            .with_annotation(
                                Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                            )
                            .with_note(typing_error.note())
                            .with_help(typing_error.help()),
                    );
                }
            };
        }
    }

    for member in members.iter() {
        match member {
            ClassLikeMember::Constant(constant) => {
                for constant_metadata in scan_class_like_constants(
                    &mut class_like_metadata,
                    constant,
                    Some(name),
                    &type_context,
                    context,
                    scope,
                ) {
                    if class_like_metadata.constants.contains_key(&constant_metadata.name) {
                        continue;
                    }

                    class_like_metadata.constants.insert(constant_metadata.name, constant_metadata);
                }
            }
            ClassLikeMember::EnumCase(enum_case) => {
                let case_metadata = scan_enum_case(enum_case, context);
                if class_like_metadata.constants.contains_key(&case_metadata.name) {
                    continue;
                }

                class_like_metadata.enum_cases.insert(case_metadata.name, case_metadata);
            }
            _ => {
                continue;
            }
        }
    }

    if class_like_metadata.kind.is_enum() {
        let enum_name_span = class_like_metadata.name_span.expect("Enum name span should be present");
        let mut name_types = vec![];
        let mut value_types = vec![];
        let backing_type = class_like_metadata.enum_type.as_ref().cloned();

        if class_like_metadata.enum_cases.len() <= MAX_ENUM_CASES_FOR_ANALYSIS {
            for (case_name, case_info) in &class_like_metadata.enum_cases {
                name_types.push(TAtomic::Scalar(TScalar::literal_string(*case_name)));

                if let Some(enum_backing_type) = &backing_type {
                    if let Some(t) = case_info.value_type.as_ref() {
                        value_types.push(t.clone());
                    } else {
                        value_types.push(enum_backing_type.clone());
                    }
                }
            }
        }

        let name_union = if name_types.is_empty() { get_string() } else { TUnion::from_vec(name_types) };

        if value_types.is_empty()
            && let Some(enum_backing_type) = &backing_type
        {
            value_types.push(enum_backing_type.clone());
        }

        let name = atom("$name");
        let flags = MetadataFlags::READONLY | MetadataFlags::HAS_DEFAULT;
        let mut property_metadata = PropertyMetadata::new(VariableIdentifier(name), flags);
        property_metadata.type_declaration_metadata = Some(TypeMetadata::new(get_string(), enum_name_span));
        property_metadata.type_metadata = Some(TypeMetadata::new(name_union, enum_name_span));

        class_like_metadata.add_property_metadata(property_metadata);

        if let Some(enum_backing_type) = backing_type {
            let value = atom("$value");

            let flags = MetadataFlags::READONLY | MetadataFlags::HAS_DEFAULT;
            let mut property_metadata = PropertyMetadata::new(VariableIdentifier(value), flags);

            property_metadata.set_type_declaration_metadata(Some(TypeMetadata::new(
                TUnion::from_vec(vec![enum_backing_type]),
                enum_name_span,
            )));

            if !value_types.is_empty() {
                property_metadata
                    .set_type_metadata(Some(TypeMetadata::new(TUnion::from_vec(value_types), enum_name_span)));
            }

            class_like_metadata.add_property_metadata(property_metadata);
        }
    }

    for member in members.iter() {
        match member {
            ClassLikeMember::TraitUse(trait_use) => {
                for trait_use in trait_use.trait_names.iter() {
                    let trait_name = context.resolved_names.get(trait_use);

                    class_like_metadata.add_used_trait(ascii_lowercase_atom(trait_name));
                }

                if let TraitUseSpecification::Concrete(specification) = &trait_use.specification {
                    for adaptation in specification.adaptations.iter() {
                        match adaptation {
                            TraitUseAdaptation::Precedence(_) => {
                                continue;
                            }
                            TraitUseAdaptation::Alias(adaptation) => {
                                let method_name = match &adaptation.method_reference {
                                    TraitUseMethodReference::Identifier(local_identifier) => &local_identifier.value,
                                    TraitUseMethodReference::Absolute(_) => {
                                        continue;
                                    }
                                };

                                if let Some(alias) = &adaptation.alias {
                                    class_like_metadata.add_trait_alias(atom(method_name), atom(alias.value));
                                }

                                if let Some(visibility) = &adaptation.visibility {
                                    let visibility = match visibility {
                                        Modifier::Public(_) => Visibility::Public,
                                        Modifier::Protected(_) => Visibility::Protected,
                                        Modifier::Private(_) => Visibility::Private,
                                        Modifier::Final(_) => {
                                            class_like_metadata.trait_final_map.insert(atom(method_name));

                                            continue;
                                        }
                                        _ => {
                                            continue;
                                        }
                                    };

                                    class_like_metadata.add_trait_visibility(atom(method_name), visibility);
                                }
                            }
                        }
                    }
                }

                let docblock = match TraitUseDocblockComment::create(context, trait_use) {
                    Ok(docblock) => docblock,
                    Err(parse_error) => {
                        class_like_metadata.issues.push(
                            Issue::error("Failed to parse trait use docblock comment.")
                                .with_code(ScanningIssueKind::MalformedDocblockComment)
                                .with_annotation(
                                    Annotation::primary(parse_error.span()).with_message(parse_error.to_string()),
                                )
                                .with_note(parse_error.note())
                                .with_help(parse_error.help()),
                        );

                        continue;
                    }
                };

                let Some(docblock) = docblock else {
                    continue;
                };

                for template_use in docblock.template_use {
                    let template_use_type = match builder::get_type_from_string(
                        &template_use.value,
                        template_use.span,
                        scope,
                        &type_context,
                        Some(name),
                    ) {
                        Ok(template_use_type) => template_use_type,
                        Err(typing_error) => {
                            class_like_metadata.issues.push(
                                Issue::error("Could not resolve the trait type in the `@use` tag.")
                                    .with_code(ScanningIssueKind::InvalidUseTag)
                                    .with_annotation(
                                        Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                                    )
                                    .with_note(typing_error.note())
                                    .with_help(typing_error.help()),
                            );

                            continue;
                        }
                    };

                    if !template_use_type.is_single() {
                        class_like_metadata.issues.push(
                            Issue::error("The `@use` tag expects a single trait type.")
                                .with_code(ScanningIssueKind::InvalidUseTag)
                                .with_annotation(
                                    Annotation::primary(template_use.span)
                                        .with_message("Union types are not allowed here."),
                                )
                                .with_note("The `@use` tag provides concrete types for generics from a trait.")
                                .with_help("Provide a single trait type, e.g., `@use MyTrait<string>`."),
                        );

                        continue;
                    }

                    let (trait_name, trait_parameters) = match template_use_type.get_single_owned() {
                        TAtomic::Reference(TReference::Symbol { name, parameters, intersection_types: None }) => {
                            (name, parameters)
                        }
                        _ => {
                            class_like_metadata.issues.push(
                                Issue::error("The `@use` tag expects a single trait type.")
                                    .with_code(ScanningIssueKind::InvalidUseTag)
                                    .with_annotation(Annotation::primary(template_use.span).with_message(
                                        "This must be a trait name, not a primitive or other complex type.",
                                    ))
                                    .with_note("The `@use` tag provides concrete types for generics from a trait.")
                                    .with_help("Provide the single trait type, e.g., `@use MyTrait<string>`."),
                            );

                            continue;
                        }
                    };

                    let lowercase_trait_name = ascii_lowercase_atom(&trait_name);
                    if !class_like_metadata.used_traits.contains(&lowercase_trait_name) {
                        class_like_metadata.issues.push(
                            Issue::error("The `@use` tag must refer to a trait that is used.")
                                .with_code(ScanningIssueKind::InvalidUseTag)
                                .with_annotation(
                                    Annotation::primary(template_use.span).with_message(format!(
                                        "The trait `{trait_name}` is not used in this class.",
                                    )),
                                )
                                .with_note("The `@use` tag is used to provide type information for the trait that is used in this class.")
                                .with_help(format!(
                                    "Ensure this class's definition includes `use {trait_name};`.",
                                )),
                        );

                        continue;
                    }

                    match trait_parameters.filter(|parameters| !parameters.is_empty()) {
                        Some(trait_parameters) => {
                            let parameters_count = trait_parameters.len();

                            class_like_metadata.template_type_uses_count.insert(lowercase_trait_name, parameters_count);
                            class_like_metadata
                                .template_extended_offsets
                                .insert(lowercase_trait_name, trait_parameters);
                        }
                        // The `@use` tag is redundant if no parameters are provided.
                        None => {
                            class_like_metadata.issues.push(
                                Issue::error("The `@use` tag must specify type parameters.")
                                    .with_code(ScanningIssueKind::InvalidUseTag)
                                    .with_annotation(
                                        Annotation::primary(template_use.span).with_message(
                                            "This tag must provide type parameters for the trait.",
                                        ),
                                    )
                                    .with_note("The `@use` tag is used to provide type information for the trait that is used in this class.")
                                    .with_help("Provide type parameters, e.g., `@use MyTrait<string>`."),
                            );

                            continue;
                        }
                    }
                }

                for template_implements in docblock.template_implements {
                    class_like_metadata.issues.push(
                        Issue::error("The `@implements` tag is not allowed in trait use.")
                            .with_code(ScanningIssueKind::InvalidUseTag)
                            .with_annotation(
                                Annotation::primary(template_implements.span)
                                    .with_message("Use `@use` for traits, not `@implements`."),
                            )
                            .with_note("The `@implements` tag is used for interface, not traits.")
                            .with_help("Use `@use` to provide type information for traits."),
                    );
                }

                for template_extends in docblock.template_extends {
                    class_like_metadata.issues.push(
                        Issue::error("The `@extends` tag is not allowed in trait use.")
                            .with_code(ScanningIssueKind::InvalidUseTag)
                            .with_annotation(
                                Annotation::primary(template_extends.span)
                                    .with_message("Use `@use` for traits, not `@extends`."),
                            )
                            .with_note("The `@extends` tag is used for classes and interfaces, not traits.")
                            .with_help("Use `@use` to provide type information for traits."),
                    );
                }
            }
            ClassLikeMember::Property(property) => {
                let properties =
                    scan_properties(property, &mut class_like_metadata, Some(name), &type_context, context, scope);

                for property_metadata in properties {
                    class_like_metadata.add_property_metadata(property_metadata);
                }
            }
            _ => {
                continue;
            }
        }
    }

    if !class_like_metadata.kind.is_trait() {
        let to_string_method = atom("__tostring");
        if class_like_metadata.methods.contains(&to_string_method) {
            class_like_metadata.add_direct_parent_interface(atom("stringable"));
        }
    }

    Some(class_like_metadata)
}
