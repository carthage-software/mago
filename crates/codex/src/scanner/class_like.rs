use mago_interner::StringIdentifier;
use mago_names::scope::NamespaceScope;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::issue::ScanningIssueKind;
use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::function_like::FunctionLikeKind;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::metadata::function_like::MethodMetadata;
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
use crate::scanner::enum_case::scan_enum_case;
use crate::scanner::function_like::scan_method;
use crate::scanner::property::scan_promoted_property;
use crate::scanner::property::scan_properties;
use crate::symbol::SymbolKind;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
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
pub fn register_anonymous_class(
    codebase: &mut CodebaseMetadata,
    class: &AnonymousClass,
    context: &mut Context<'_>,
    scope: &mut NamespaceScope,
) -> Option<(StringIdentifier, TemplateConstraintList)> {
    let span = class.span();
    let name = context.interner.intern(format!(
        "class@anonymous:{}-{}:{}",
        span.start.source.0.value(),
        span.start.offset,
        span.end.offset,
    ));

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
        .get_template_types()
        .iter()
        .map(|(name, definition)| (context.interner.lookup(name).to_string(), definition.clone()))
        .collect::<TemplateConstraintList>();

    codebase.class_likes.insert(name, class_like_metadata);

    Some((name, template_resolution_context))
}

#[inline]
pub fn register_class(
    codebase: &mut CodebaseMetadata,
    class: &Class,
    context: &mut Context<'_>,
    scope: &mut NamespaceScope,
) -> Option<(StringIdentifier, TemplateConstraintList)> {
    let class_like_metadata = scan_class_like(
        codebase,
        *context.resolved_names.get(&class.name),
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
        .get_template_types()
        .iter()
        .map(|(name, definition)| (context.interner.lookup(name).to_string(), definition.clone()))
        .collect::<TemplateConstraintList>();

    let name = class_like_metadata.name;

    codebase.class_likes.insert(name, class_like_metadata);

    Some((name, template_resolution_context))
}

#[inline]
pub fn register_interface(
    codebase: &mut CodebaseMetadata,
    interface: &Interface,
    context: &mut Context<'_>,
    scope: &mut NamespaceScope,
) -> Option<(StringIdentifier, TemplateConstraintList)> {
    let class_like_metadata = scan_class_like(
        codebase,
        *context.resolved_names.get(&interface.name),
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
        .get_template_types()
        .iter()
        .map(|(name, definition)| (context.interner.lookup(name).to_string(), definition.clone()))
        .collect::<TemplateConstraintList>();

    let name = class_like_metadata.name;

    codebase.class_likes.insert(name, class_like_metadata);

    Some((name, template_resolution_context))
}

#[inline]
pub fn register_trait(
    codebase: &mut CodebaseMetadata,
    r#trait: &Trait,
    context: &mut Context<'_>,
    scope: &mut NamespaceScope,
) -> Option<(StringIdentifier, TemplateConstraintList)> {
    let class_like_metadata = scan_class_like(
        codebase,
        *context.resolved_names.get(&r#trait.name),
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
        .get_template_types()
        .iter()
        .map(|(name, definition)| (context.interner.lookup(name).to_string(), definition.clone()))
        .collect::<TemplateConstraintList>();

    let name = class_like_metadata.name;

    codebase.class_likes.insert(name, class_like_metadata);

    Some((name, template_resolution_context))
}

#[inline]
pub fn register_enum(
    codebase: &mut CodebaseMetadata,
    r#enum: &Enum,
    context: &mut Context<'_>,
    scope: &mut NamespaceScope,
) -> Option<(StringIdentifier, TemplateConstraintList)> {
    let class_like_metadata = scan_class_like(
        codebase,
        *context.resolved_names.get(&r#enum.name),
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
        .get_template_types()
        .iter()
        .map(|(name, definition)| (context.interner.lookup(name).to_string(), definition.clone()))
        .collect::<TemplateConstraintList>();

    let name = class_like_metadata.name;

    codebase.class_likes.insert(name, class_like_metadata);

    Some((name, template_resolution_context))
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn scan_class_like(
    codebase: &mut CodebaseMetadata,
    name: StringIdentifier,
    kind: SymbolKind,
    name_span: Option<Span>,
    span: Span,
    attribute_lists: &Sequence<AttributeList>,
    modifiers: Option<&Sequence<Modifier>>,
    members: &Sequence<ClassLikeMember>,
    extends: Option<&Extends>,
    implements: Option<&Implements>,
    enum_type: Option<&EnumBackingTypeHint>,
    context: &mut Context<'_>,
    scope: &mut NamespaceScope,
) -> Option<ClassLikeMetadata> {
    let original_name = name;
    let name = context.interner.lowered(&original_name);

    if codebase.class_likes.contains_key(&name) {
        return None;
    }

    let mut class_like_metadata = ClassLikeMetadata::new(name, original_name, span, name_span)
        .with_attributes(scan_attribute_lists(attribute_lists, context))
        .with_enum_type(match enum_type {
            Some(EnumBackingTypeHint { hint: Hint::String(_), .. }) => Some(TAtomic::Scalar(TScalar::string())),
            Some(EnumBackingTypeHint { hint: Hint::Integer(_), .. }) => Some(TAtomic::Scalar(TScalar::int())),
            _ => None,
        });

    if kind.is_class() {
        class_like_metadata.set_attribute_flags(get_attribute_flags(name, attribute_lists, context));
    }

    match kind {
        SymbolKind::Class => {
            class_like_metadata = class_like_metadata
                .with_kind(kind)
                .with_is_final(modifiers.is_some_and(|m| m.contains_final()))
                .with_is_abstract(modifiers.is_some_and(|m| m.contains_abstract()))
                .with_is_readonly(modifiers.is_some_and(|m| m.contains_readonly()));

            codebase.symbols.add_class_name(name);

            if let Some(extended_class) = extends.and_then(|e| e.types.first()) {
                let parent_name = context.resolved_names.get(extended_class);
                let parent_name = context.interner.lowered(parent_name);

                class_like_metadata = class_like_metadata.with_direct_parent_class(Some(parent_name));
            }
        }
        SymbolKind::Enum => {
            class_like_metadata = class_like_metadata.with_kind(kind).with_is_final(true);

            if enum_type.is_some() {
                let backed_enum_interface = context.interner.intern("backedenum");
                let from_method = context.interner.intern("from");
                let try_from_method = context.interner.intern("tryfrom");

                class_like_metadata.add_direct_parent_interface(backed_enum_interface);
                class_like_metadata.add_declaring_method_id(from_method, backed_enum_interface);
                class_like_metadata.add_declaring_method_id(try_from_method, backed_enum_interface);
            }

            let unit_enum_interface = context.interner.intern("unitenum");
            let cases_method = context.interner.intern("cases");

            class_like_metadata.add_direct_parent_interface(unit_enum_interface);
            class_like_metadata.add_declaring_method_id(cases_method, unit_enum_interface);

            codebase.symbols.add_enum_name(name);
        }
        SymbolKind::Trait => {
            class_like_metadata = class_like_metadata.with_kind(kind);

            codebase.symbols.add_trait_name(name);
        }
        SymbolKind::Interface => {
            class_like_metadata = class_like_metadata.with_is_abstract(true).with_kind(kind);

            codebase.symbols.add_interface_name(name);

            if let Some(extends) = extends {
                for extended_interface in extends.types.iter() {
                    let parent_name = context.resolved_names.get(extended_interface);
                    let parent_name = context.interner.lowered(parent_name);

                    class_like_metadata.add_direct_parent_interface(parent_name);
                }
            }
        }
    };

    if (class_like_metadata.is_class() || class_like_metadata.is_enum())
        && let Some(implemented_interfaces) = implements
    {
        for interface_name in implemented_interfaces.types.iter() {
            let interface_name = context.resolved_names.get(interface_name);
            let interface_name = context.interner.lowered(interface_name);

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
        class_like_metadata = class_like_metadata
            .with_has_sealed_methods(docblock.has_sealed_methods)
            .with_has_sealed_properties(docblock.has_sealed_properties);

        class_like_metadata.is_enum_interface = class_like_metadata.is_interface() && docblock.is_enum_interface;
        class_like_metadata.is_final |= docblock.is_final;
        class_like_metadata.is_deprecated |= docblock.is_deprecated;
        class_like_metadata.is_immutable |= docblock.is_immutable;
        class_like_metadata.is_internal |= docblock.is_internal;
        class_like_metadata.is_mutation_free |= docblock.is_mutation_free;
        class_like_metadata.is_external_mutation_free |= docblock.is_external_mutation_free;
        class_like_metadata.allows_private_mutation |= docblock.allows_private_mutation;
        class_like_metadata.has_consistent_constructor |= docblock.has_consistent_constructor;
        class_like_metadata.has_consistent_templates |= docblock.has_consistent_templates;

        for (i, template) in docblock.templates.iter().enumerate() {
            let template_name = context.interner.intern(&template.name);
            let template_as_type = if let Some(type_string) = &template.type_string {
                match builder::get_type_from_string(
                    &type_string.value,
                    type_string.span,
                    scope,
                    &type_context,
                    Some(&name),
                    context.interner,
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
            type_context = type_context.with_template_definition(template.name.clone(), definition);

            let variance = if template.covariant {
                Variance::Covariant
            } else if template.contravariant {
                Variance::Contravariant
            } else {
                Variance::Invariant
            };

            if variance.is_readonly() {
                class_like_metadata.add_template_readonly(template_name);
            }

            class_like_metadata.add_template_variance_parameter(i, variance);
        }

        for extended_type in docblock.template_extends {
            let extended_union = match builder::get_type_from_string(
                &extended_type.value,
                extended_type.span,
                scope,
                &type_context,
                Some(&name),
                context.interner,
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

            let parent_name_str = context.interner.lookup(&parent_name);
            let parent_name = context.interner.lowered(&parent_name);

            let has_parent = if class_like_metadata.is_interface() {
                class_like_metadata.has_parent_interface(&parent_name)
            } else {
                class_like_metadata.has_parent_class(&parent_name)
            };

            if !has_parent {
                class_like_metadata.issues.push(
                    Issue::error("`@extends` tag must refer to a direct parent class or interface.")
                        .with_code(ScanningIssueKind::InvalidExtendsTag)
                        .with_annotation(Annotation::primary(extended_type.span).with_message(format!(
                            "The class `{parent_name_str}` is not a direct parent."
                        )))
                        .with_note("The `@extends` tag is used to provide type information for the class or interface that is directly extended.")
                        .with_help(format!("Ensure this type's definition includes `extends {parent_name_str}`.")),
                );

                continue;
            }

            if let Some(extended_parent_parameters) = parent_parameters {
                class_like_metadata.template_type_extends_count.insert(parent_name, extended_parent_parameters.len());
                class_like_metadata.add_template_extended_offset(parent_name, extended_parent_parameters);
            }
        }

        for implemented_type in docblock.template_implements {
            let implemented_union = match builder::get_type_from_string(
                &implemented_type.value,
                implemented_type.span,
                scope,
                &type_context,
                Some(&name),
                context.interner,
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
                    let atomic_str = atomic.get_id(Some(context.interner));

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

            let parent_name_str = context.interner.lookup(&parent_name);
            let parent_name = context.interner.lowered(&parent_name);

            if !class_like_metadata.has_parent_interface(&context.interner.lowered(&parent_name)) {
                class_like_metadata.issues.push(
                    Issue::error("The `@implements` tag must refer to a direct parent interface.")
                        .with_code(ScanningIssueKind::InvalidImplementsTag)
                        .with_annotation(Annotation::primary(implemented_type.span).with_message(format!(
                            "The interface `{parent_name_str}` is not a direct parent."
                        )))
                        .with_note("The `@implements` tag is used to provide type information for the interface that is directly implemented.")
                        .with_help(format!("Ensure this type's definition includes `implements {parent_name_str}`.")),
                );

                continue;
            }

            if let Some(impl_parent_parameters) = parent_parameters {
                class_like_metadata.template_type_implements_count.insert(parent_name, impl_parent_parameters.len());
                class_like_metadata.add_template_extended_offset(parent_name, impl_parent_parameters);
            }
        }

        for require_extend in docblock.require_extends {
            let required_union = match builder::get_type_from_string(
                &require_extend.value,
                require_extend.span,
                scope,
                &type_context,
                Some(&name),
                context.interner,
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
                TAtomic::Object(TObject::Named(named_object)) => {
                    if named_object.is_intersection() {
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

                    (named_object.name, named_object.type_parameters)
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

            class_like_metadata.add_require_extend(context.interner.lowered(&required_name));
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
                Some(&name),
                context.interner,
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
                TAtomic::Object(TObject::Named(named_object)) => {
                    if named_object.is_intersection() {
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

                    (named_object.name, named_object.type_parameters)
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

            class_like_metadata.add_require_implement(context.interner.lowered(&required_name));
            if let Some(required_parameters) = required_parameters {
                class_like_metadata.add_template_extended_offset(required_name, required_parameters);
            }
        }

        if let Some(inheritors) = docblock.inheritors {
            match builder::get_type_from_string(
                &inheritors.value,
                inheritors.span,
                scope,
                &type_context,
                Some(&name),
                context.interner,
            ) {
                Ok(inheritors_union) => {
                    for inheritor in inheritors_union.types {
                        match inheritor {
                            TAtomic::Reference(TReference::Symbol {
                                name,
                                parameters: None,
                                intersection_types: None,
                            }) => {
                                class_like_metadata
                                    .permitted_inheritors
                                    .get_or_insert_default()
                                    .insert(context.interner.lowered(&name));
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
                for constant_metadata in scan_class_like_constants(&mut class_like_metadata, constant, context) {
                    let constant_name = constant_metadata.get_name();
                    if class_like_metadata.has_constant(constant_name) {
                        continue;
                    }

                    class_like_metadata.add_constant(*constant_name, constant_metadata);
                }
            }
            ClassLikeMember::EnumCase(enum_case) => {
                let case_metadata = scan_enum_case(enum_case, context);
                let case_name = case_metadata.get_name();
                if class_like_metadata.has_enum_case(case_name) {
                    continue;
                }

                class_like_metadata.add_enum_case(*case_name, case_metadata);
            }
            _ => {
                continue;
            }
        }
    }

    if class_like_metadata.is_enum() {
        let enum_name_span = class_like_metadata.get_name_span().expect("Enum name span should be present");
        let mut name_types = vec![];
        let mut value_types = vec![];
        let backing_type = class_like_metadata.enum_type.as_ref().cloned();

        for (case_name, case_info) in class_like_metadata.get_enum_cases() {
            name_types.push(TAtomic::Scalar(TScalar::literal_string(context.interner.lookup(case_name).to_string())));

            if let Some(enum_backing_type) = &backing_type {
                if let Some(t) = case_info.get_value_type() {
                    value_types.push(t.clone());
                } else {
                    value_types.push(enum_backing_type.clone());
                }
            }
        }

        if !name_types.is_empty() {
            let name = context.interner.intern("$name");
            let mut property_metadata = PropertyMetadata::new(VariableIdentifier(name));
            property_metadata.is_readonly = true;
            property_metadata.has_default = true;
            property_metadata.type_declaration_metadata = Some(TypeMetadata::new(get_string(), enum_name_span));
            property_metadata.type_metadata = Some(TypeMetadata::new(TUnion::new(name_types), enum_name_span));

            class_like_metadata.add_property_metadata(property_metadata);
        }

        if let Some(enum_backing_type) = backing_type
            && !value_types.is_empty()
        {
            let value = context.interner.intern("$value");

            let mut property_metadata = PropertyMetadata::new(VariableIdentifier(value));
            property_metadata.is_readonly = true;
            property_metadata.has_default = true;
            property_metadata.type_declaration_metadata =
                Some(TypeMetadata::new(TUnion::new(vec![enum_backing_type]), enum_name_span));
            property_metadata.type_metadata = Some(TypeMetadata::new(TUnion::new(value_types), enum_name_span));

            class_like_metadata.add_property_metadata(property_metadata);
        }
    }

    let clone_name_id = context.interner.intern("__clone");
    let mut has_constructor = false;

    for member in members.iter() {
        match member {
            ClassLikeMember::TraitUse(trait_use) => {
                for trait_use in trait_use.trait_names.iter() {
                    let trait_name = context.resolved_names.get(trait_use);

                    class_like_metadata.add_used_trait(context.interner.lowered(trait_name));
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
                                    class_like_metadata.add_trait_alias(*method_name, alias.value);
                                }

                                if let Some(visibility) = &adaptation.visibility {
                                    let visibility = match visibility {
                                        Modifier::Public(_) => Visibility::Public,
                                        Modifier::Protected(_) => Visibility::Protected,
                                        Modifier::Private(_) => Visibility::Private,
                                        Modifier::Final(_) => {
                                            class_like_metadata.add_trait_final(*method_name);

                                            continue;
                                        }
                                        _ => {
                                            continue;
                                        }
                                    };

                                    class_like_metadata.add_trait_visibility(*method_name, visibility);
                                }
                            }
                        }
                    }
                }
            }
            ClassLikeMember::Property(property) => {
                let properties =
                    scan_properties(property, &mut class_like_metadata, Some(&name), &type_context, context, scope);

                for property_metadata in properties {
                    class_like_metadata.add_property_metadata(property_metadata);
                }
            }
            ClassLikeMember::Method(method) => {
                let name = context.interner.lowered(&method.name.value);
                if class_like_metadata.has_method(&name) {
                    continue;
                }

                let method_id = (class_like_metadata.name, name);
                let type_resolution = if method.is_static() { None } else { Some(type_context.clone()) };

                let function_like_metadata =
                    scan_method(method_id, method, &class_like_metadata, context, scope, type_resolution);
                let Some(method_metadata) = &function_like_metadata.get_method_metadata() else {
                    unreachable!("Method info should be present for method.",);
                };

                let mut is_constructor = false;
                let mut is_clone = false;
                if method_metadata.is_constructor() {
                    is_constructor = true;
                    has_constructor = true;

                    for (index, param) in method.parameter_list.parameters.iter().enumerate() {
                        if !param.is_promoted_property() {
                            continue;
                        }

                        let Some(parameter_info) = function_like_metadata.get_parameters().get(index) else {
                            continue;
                        };

                        let property_metadata =
                            scan_promoted_property(param, parameter_info, &class_like_metadata, context);

                        class_like_metadata.add_property_metadata(property_metadata);
                    }
                } else {
                    is_clone = name == clone_name_id;
                }

                class_like_metadata.add_method(name);
                class_like_metadata.add_declaring_method_id(name, class_like_metadata.name);
                if !method_metadata.get_visibility().is_private()
                    || is_constructor
                    || is_clone
                    || class_like_metadata.is_trait()
                {
                    class_like_metadata.add_inheritable_method_id(name, class_like_metadata.name);
                }

                if method_metadata.is_final() && is_constructor {
                    class_like_metadata.set_has_consistent_constructor(true);
                }

                codebase.function_likes.insert(method_id, function_like_metadata);
            }
            _ => {
                continue;
            }
        }
    }

    if !class_like_metadata.is_trait() {
        let to_string_method = context.interner.intern("__tostring");
        if class_like_metadata.methods.contains(&to_string_method) {
            class_like_metadata.add_direct_parent_interface(context.interner.intern("stringable"));
        }
    }

    if class_like_metadata.has_consistent_constructor() && !has_constructor {
        let constructor_name = context.interner.intern("__construct");

        let mut function_like_metadata =
            FunctionLikeMetadata::new(FunctionLikeKind::Method, class_like_metadata.get_span());

        function_like_metadata.method_metadata = Some(MethodMetadata::new(Visibility::Public));
        function_like_metadata.is_mutation_free = true;
        function_like_metadata.is_external_mutation_free = true;

        class_like_metadata.add_method(constructor_name);
        class_like_metadata.add_declaring_method_id(constructor_name, class_like_metadata.name);
        class_like_metadata.add_inheritable_method_id(constructor_name, class_like_metadata.name);

        codebase.function_likes.insert((class_like_metadata.name, constructor_name), function_like_metadata);
    }

    Some(class_like_metadata)
}
