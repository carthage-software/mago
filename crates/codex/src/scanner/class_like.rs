use mago_allocator::Arena;
use mago_word::Word;
use std::collections::BTreeMap;

use mago_names::kind::NameKind;
use mago_names::scope::NamespaceScope;
use mago_phpdoc_syntax::cst::TagValue;
use mago_phpdoc_syntax::cst::Visibility as DocblockVisibility;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::AnonymousClass;
use mago_syntax::cst::AttributeList;
use mago_syntax::cst::Class;
use mago_syntax::cst::ClassLikeMember;
use mago_syntax::cst::Enum;
use mago_syntax::cst::EnumBackingTypeHint;
use mago_syntax::cst::Extends;
use mago_syntax::cst::Hint;
use mago_syntax::cst::Implements;
use mago_syntax::cst::Interface;
use mago_syntax::cst::Modifier;
use mago_syntax::cst::ModifierSequenceExt;
use mago_syntax::cst::Sequence;
use mago_syntax::cst::Trait;
use mago_syntax::cst::TraitUseAdaptation;
use mago_syntax::cst::TraitUseMethodReference;
use mago_syntax::cst::TraitUseSpecification;
use mago_word::WordMap;
use mago_word::WordSet;
use mago_word::ascii_lowercase_word;
use mago_word::word;

use crate::consts::MAX_ENUM_CASES_FOR_ANALYSIS;
use crate::get_anonymous_class_name;
use crate::identifier::method::MethodIdentifier;
use crate::issue::ScanningIssueKind;
use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::class_like::TemplateTypes;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::function_like::FunctionLikeKind;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::metadata::function_like::MethodMetadata;
use crate::metadata::parameter::FunctionLikeParameterMetadata;
use crate::metadata::property::PropertyMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::GenericParent;
use crate::misc::VariableIdentifier;
use crate::scanner::Context;
use crate::scanner::TemplateConstraintList;
use crate::scanner::attribute::get_attribute_flags;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::class_like_constant::scan_class_like_constants;
use crate::scanner::docblock::parse_docblock;
use crate::scanner::enum_case::scan_enum_case;
use crate::scanner::property::scan_properties;
use crate::scanner::ttype::get_type_metadata_from_type;
use crate::scanner::version_claim::evaluate_version_attributes;
use crate::symbol::SymbolKind;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::r#enum::TEnum;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::builder;
use crate::ttype::get_list;
use crate::ttype::get_mixed;
use crate::ttype::get_named_object;
use crate::ttype::get_non_empty_list;
use crate::ttype::get_string;
use crate::ttype::resolution::TypeResolutionContext;
use crate::ttype::template::GenericTemplate;
use crate::ttype::template::variance::Variance;
use crate::ttype::union::TUnion;
use crate::visibility::Visibility;

/// Return type for class-like registration functions.
/// Returns the name, template constraints, local type aliases, and imported type aliases.
type ClassLikeRegistration = Option<(Word, TemplateConstraintList, WordSet, WordMap<(Word, Word)>)>;

#[inline]
pub fn register_anonymous_class<'arena, A>(
    codebase: &mut CodebaseMetadata,
    class: &'arena AnonymousClass<'arena>,
    context: &mut Context<'_, 'arena, A>,
    scope: &mut NamespaceScope,
) -> ClassLikeRegistration
where
    A: Arena,
{
    let span = class.span();
    let name = get_anonymous_class_name(context.file, span);

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

    let name = class_like_metadata.name;
    let type_aliases = class_like_metadata.type_aliases.keys().copied().collect::<WordSet>();
    let imported_aliases = class_like_metadata
        .imported_type_aliases
        .iter()
        .map(|(local_name, (source_class, original_name, _span))| (*local_name, (*source_class, *original_name)))
        .collect::<WordMap<(Word, Word)>>();

    codebase.class_likes.insert(name, class_like_metadata);

    Some((name, template_resolution_context, type_aliases, imported_aliases))
}

#[inline]
pub fn register_class<'arena, A>(
    codebase: &mut CodebaseMetadata,
    class: &'arena Class<'arena>,
    context: &mut Context<'_, 'arena, A>,
    scope: &mut NamespaceScope,
) -> ClassLikeRegistration
where
    A: Arena,
{
    let class_like_metadata = scan_class_like(
        codebase,
        word(context.resolved_names.get(&class.name)),
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
    let type_aliases = class_like_metadata.type_aliases.keys().copied().collect::<WordSet>();
    let imported_aliases = class_like_metadata
        .imported_type_aliases
        .iter()
        .map(|(local_name, (source_class, original_name, _span))| (*local_name, (*source_class, *original_name)))
        .collect::<WordMap<(Word, Word)>>();

    codebase.class_likes.insert(name, class_like_metadata);

    Some((name, template_resolution_context, type_aliases, imported_aliases))
}

#[inline]
pub fn register_interface<'arena, A>(
    codebase: &mut CodebaseMetadata,
    interface: &'arena Interface<'arena>,
    context: &mut Context<'_, 'arena, A>,
    scope: &mut NamespaceScope,
) -> ClassLikeRegistration
where
    A: Arena,
{
    let class_like_metadata = scan_class_like(
        codebase,
        word(context.resolved_names.get(&interface.name)),
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
    let type_aliases = class_like_metadata.type_aliases.keys().copied().collect::<WordSet>();
    let imported_aliases = class_like_metadata
        .imported_type_aliases
        .iter()
        .map(|(local_name, (source_class, original_name, _span))| (*local_name, (*source_class, *original_name)))
        .collect::<WordMap<(Word, Word)>>();

    codebase.class_likes.insert(name, class_like_metadata);

    Some((name, template_resolution_context, type_aliases, imported_aliases))
}

#[inline]
pub fn register_trait<'arena, A>(
    codebase: &mut CodebaseMetadata,
    r#trait: &'arena Trait<'arena>,
    context: &mut Context<'_, 'arena, A>,
    scope: &mut NamespaceScope,
) -> ClassLikeRegistration
where
    A: Arena,
{
    let class_like_metadata = scan_class_like(
        codebase,
        word(context.resolved_names.get(&r#trait.name)),
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
    let type_aliases = class_like_metadata.type_aliases.keys().copied().collect::<WordSet>();
    let imported_aliases = class_like_metadata
        .imported_type_aliases
        .iter()
        .map(|(local_name, (source_class, original_name, _span))| (*local_name, (*source_class, *original_name)))
        .collect::<WordMap<(Word, Word)>>();

    codebase.class_likes.insert(name, class_like_metadata);

    Some((name, template_resolution_context, type_aliases, imported_aliases))
}

#[inline]
pub fn register_enum<'arena, A>(
    codebase: &mut CodebaseMetadata,
    r#enum: &'arena Enum<'arena>,
    context: &mut Context<'_, 'arena, A>,
    scope: &mut NamespaceScope,
) -> ClassLikeRegistration
where
    A: Arena,
{
    let class_like_metadata = scan_class_like(
        codebase,
        word(context.resolved_names.get(&r#enum.name)),
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
    let type_aliases = class_like_metadata.type_aliases.keys().copied().collect::<WordSet>();
    let imported_aliases = class_like_metadata
        .imported_type_aliases
        .iter()
        .map(|(local_name, (source_class, original_name, _span))| (*local_name, (*source_class, *original_name)))
        .collect::<WordMap<(Word, Word)>>();

    codebase.class_likes.insert(name, class_like_metadata);

    Some((name, template_resolution_context, type_aliases, imported_aliases))
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn scan_class_like<'arena, A>(
    codebase: &mut CodebaseMetadata,
    name: Word,
    kind: SymbolKind,
    name_span: Option<Span>,
    span: Span,
    attribute_lists: &'arena Sequence<'arena, AttributeList<'arena>>,
    modifiers: Option<&'arena Sequence<Modifier<'arena>>>,
    members: &'arena Sequence<ClassLikeMember<'arena>>,
    extends: Option<&'arena Extends<'arena>>,
    implements: Option<&'arena Implements<'arena>>,
    enum_type: Option<&'arena EnumBackingTypeHint<'arena>>,
    context: &mut Context<'_, 'arena, A>,
    scope: &mut NamespaceScope,
) -> Option<ClassLikeMetadata>
where
    A: Arena,
{
    let original_name = name;
    let name = ascii_lowercase_word(original_name.as_bytes());

    if codebase.class_likes.contains_key(&name) {
        return None;
    }

    let verdict = evaluate_version_attributes(attribute_lists, context, context.php_version);

    let flags = MetadataFlags::origin_flags(context.file.file_type);

    let mut class_like_metadata = ClassLikeMetadata::new(name, original_name, span, name_span, flags);
    class_like_metadata.version_constraint = verdict.constraint;

    class_like_metadata.attributes = scan_attribute_lists(attribute_lists, context);
    class_like_metadata.enum_type = match enum_type {
        Some(EnumBackingTypeHint { hint: Hint::String(_), .. }) => Some(TAtomic::Scalar(TScalar::string())),
        Some(EnumBackingTypeHint { hint: Hint::Integer(_), .. }) => Some(TAtomic::Scalar(TScalar::int())),
        _ => None,
    };

    if kind.is_class() {
        class_like_metadata.attribute_flags =
            get_attribute_flags(name, attribute_lists, context, scope, Some(original_name));
    }

    class_like_metadata.kind = kind;

    match kind {
        SymbolKind::Class => {
            if modifiers.is_some_and(Sequence::contains_final) {
                class_like_metadata.flags |= MetadataFlags::FINAL;
            }

            if modifiers.is_some_and(Sequence::contains_abstract) {
                class_like_metadata.flags |= MetadataFlags::ABSTRACT;
            }

            if modifiers.is_some_and(Sequence::contains_readonly) {
                class_like_metadata.flags |= MetadataFlags::READONLY;
            }

            codebase.symbols.add_class_name(name);

            if let Some(extended_class) = extends.and_then(|e| e.types.first()) {
                let parent_name = context.resolved_names.get(extended_class);
                let parent_name = ascii_lowercase_word(parent_name);

                class_like_metadata.direct_parent_class = Some(parent_name);
                class_like_metadata.all_parent_classes.insert(parent_name);
            }
        }
        SymbolKind::Enum => {
            class_like_metadata.flags |= MetadataFlags::FINAL;

            match enum_type {
                Some(backing_type) => {
                    if backing_type.hint.is_string() {
                        class_like_metadata
                            .add_direct_parent_interface(word("__internal_do_not_use__stringbackedenum"));
                    } else if backing_type.hint.is_int() {
                        class_like_metadata.add_direct_parent_interface(word("__internal_do_not_use__intbackedenum"));
                    } else {
                        class_like_metadata.add_direct_parent_interface(word("backedenum"));
                    }
                }
                None => {
                    class_like_metadata.add_direct_parent_interface(word("unitenum"));
                }
            }

            codebase.symbols.add_enum_name(name);

            create_enum_methods(codebase, &mut class_like_metadata, span);
        }
        SymbolKind::Trait => {
            codebase.symbols.add_trait_name(name);
        }
        SymbolKind::Interface => {
            class_like_metadata.flags |= MetadataFlags::ABSTRACT;

            codebase.symbols.add_interface_name(name);

            if let Some(extends) = extends {
                for extended_interface in &extends.types {
                    let parent_name = context.resolved_names.get(extended_interface);
                    let parent_name = ascii_lowercase_word(parent_name);

                    class_like_metadata.add_direct_parent_interface(parent_name);
                }
            }
        }
    }

    if (class_like_metadata.kind.is_class() || class_like_metadata.kind.is_enum())
        && let Some(implemented_interfaces) = implements
    {
        for interface_name in &implemented_interfaces.types {
            let interface_name = context.resolved_names.get(interface_name);
            let interface_name = ascii_lowercase_word(interface_name);

            class_like_metadata.add_direct_parent_interface(interface_name);
        }
    }

    let mut type_context = TypeResolutionContext::new();
    let document = parse_docblock(context, span);

    if let Some(document) = document {
        for parse_error in document.errors {
            class_like_metadata.issues.push(
                Issue::error("Failed to parse class-like docblock comment.")
                    .with_code(ScanningIssueKind::MalformedDocblockComment)
                    .with_annotation(Annotation::primary(parse_error.span()).with_message(parse_error.to_string()))
                    .with_note(parse_error.note())
                    .with_help(parse_error.help()),
            );
        }

        for tag in document.tags() {
            match &tag.value {
                TagValue::EnumInterface(_) => {
                    if class_like_metadata.kind.is_interface() {
                        class_like_metadata.flags |= MetadataFlags::ENUM_INTERFACE;
                    }
                }
                TagValue::Final(_) => {
                    class_like_metadata.flags |= MetadataFlags::FINAL;
                }
                TagValue::Deprecated(_) => {
                    class_like_metadata.flags |= MetadataFlags::DEPRECATED;
                }
                TagValue::NotDeprecated(_) => {
                    class_like_metadata.flags.set(MetadataFlags::DEPRECATED, false);
                }
                TagValue::Internal(_) => {
                    class_like_metadata.flags |= MetadataFlags::INTERNAL;
                }
                TagValue::Experimental(_) => {
                    class_like_metadata.flags |= MetadataFlags::EXPERIMENTAL;
                }
                TagValue::Api(_) => {
                    class_like_metadata.flags |= MetadataFlags::API;
                }
                TagValue::ConsistentConstructor(_) => {
                    class_like_metadata.flags |= MetadataFlags::CONSISTENT_CONSTRUCTOR;
                }
                TagValue::ConsistentTemplates(_) => {
                    class_like_metadata.flags |= MetadataFlags::CONSISTENT_TEMPLATES;
                }
                TagValue::SealProperties(_) => {
                    class_like_metadata.has_sealed_properties = Some(true);
                }
                TagValue::NoSealProperties(_) => {
                    class_like_metadata.has_sealed_properties = Some(false);
                }
                TagValue::SealMethods(_) => {
                    class_like_metadata.has_sealed_methods = Some(true);
                }
                TagValue::NoSealMethods(_) => {
                    class_like_metadata.has_sealed_methods = Some(false);
                }
                _ => {}
            }
        }

        for tag in document.tags() {
            let TagValue::TypeAliasImport(imported_type_alias) = &tag.value else {
                continue;
            };

            let fqcn =
                ascii_lowercase_word(&scope.resolve_str(NameKind::Default, imported_type_alias.imported_from.value).0);
            let type_name = word(imported_type_alias.imported_alias.value);
            let alias = imported_type_alias.imported_as.as_ref().map_or(type_name, |a| word(a.local.value));

            if fqcn == name {
                continue;
            }

            type_context = type_context.with_imported_type_alias(alias, fqcn, type_name);
        }

        for tag in document.tags() {
            let TagValue::TypeAlias(type_alias) = &tag.value else {
                continue;
            };

            type_context = type_context.with_type_alias(word(type_alias.alias.value));
        }

        for tag in document.tags() {
            if let TagValue::Template(template) = &tag.value {
                scope.add(NameKind::Default, template.name.value, &(None as Option<&str>));
            }
        }

        let mut template_variance = Vec::new();
        for tag in document.tags() {
            let TagValue::Template(template) = &tag.value else {
                continue;
            };

            let template_name = word(template.name.value);
            let template_as_type = if let Some(bound) = &template.bound {
                match builder::get_union_from_type(bound.r#type, scope, &type_context, Some(original_name)) {
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

            let template_default = if let Some(default) = &template.default {
                match builder::get_union_from_type(default.r#type, scope, &type_context, Some(original_name)) {
                    Ok(tunion) => Some(tunion),
                    Err(typing_error) => {
                        class_like_metadata.issues.push(
                            Issue::error("Could not resolve the default type for the `@template` tag.")
                                .with_code(ScanningIssueKind::InvalidTemplateTag)
                                .with_annotation(
                                    Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                                )
                                .with_note(typing_error.note())
                                .with_help(typing_error.help()),
                        );

                        None
                    }
                }
            } else {
                None
            };

            let definition =
                GenericTemplate::new(GenericParent::ClassLike(name), template_as_type).with_default(template_default);

            class_like_metadata.add_template_type(template_name, definition.clone());
            type_context = type_context.with_template_definition(template_name, vec![definition]);

            let variance = Variance::from(template.variance);
            if variance.is_readonly() {
                class_like_metadata.template_readonly.insert(template_name);
            }

            template_variance.push(variance);
        }

        class_like_metadata.set_template_variance(template_variance);

        // Aliases were already registered in `type_context` above; here we
        // record them on the class metadata and report any diagnostics.
        for tag in document.tags() {
            let TagValue::TypeAliasImport(imported_type_alias) = &tag.value else {
                continue;
            };

            let fqcn =
                ascii_lowercase_word(&scope.resolve_str(NameKind::Default, imported_type_alias.imported_from.value).0);
            let type_name = word(imported_type_alias.imported_alias.value);
            let alias = imported_type_alias.imported_as.as_ref().map_or(type_name, |a| word(a.local.value));

            if fqcn == name {
                class_like_metadata.issues.push(
                    Issue::help("Type alias is importing from itself, which is unnecessary.")
                        .with_code(ScanningIssueKind::CircularTypeImport)
                        .with_annotation(
                            Annotation::primary(imported_type_alias.span())
                                .with_message(format!("Type alias `{type_name}` is already defined in this class")),
                        )
                        .with_help("Remove this import statement as the type is already available locally."),
                );

                continue;
            }

            class_like_metadata.imported_type_aliases.insert(alias, (fqcn, type_name, imported_type_alias.span()));
        }

        for tag in document.tags() {
            let TagValue::TypeAlias(type_alias) = &tag.value else {
                continue;
            };

            let alias_name = word(type_alias.alias.value);
            match get_type_metadata_from_type(type_alias.r#type, Some(original_name), &type_context, scope) {
                Ok(type_metadata) => {
                    class_like_metadata.type_aliases.insert(alias_name, type_metadata);
                }
                Err(typing_error) => {
                    class_like_metadata.issues.push(
                        Issue::error("Could not resolve the type in the `@type` tag.")
                            .with_code(ScanningIssueKind::InvalidTypeTag)
                            .with_annotation(
                                Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                            )
                            .with_note(typing_error.note())
                            .with_help(typing_error.help()),
                    );
                }
            }
        }

        for tag in document.tags() {
            let TagValue::Extends(extended_type) = &tag.value else {
                continue;
            };

            let extended_union =
                match builder::get_union_from_type(extended_type.r#type, scope, &type_context, Some(original_name)) {
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
                            Annotation::primary(extended_type.r#type.span())
                                .with_message("Union types are not allowed here."),
                        )
                        .with_note("The `@extends` tag provides concrete types for generics from a direct parent type.")
                        .with_help("Provide a single parent type, e.g., `@extends Box<string>`."),
                );

                continue;
            }

            let TAtomic::Reference(TReference::Symbol {
                name: parent_name,
                parameters: parent_parameters,
                intersection_types: None,
                ..
            }) = extended_union.get_single_owned()
            else {
                class_like_metadata.issues.push(
                    Issue::error("The `@extends` tag expects a generic class type.")
                        .with_code(ScanningIssueKind::InvalidExtendsTag)
                        .with_annotation(
                            Annotation::primary(extended_type.r#type.span())
                                .with_message("This must be a class name, not a primitive or other complex type."),
                        )
                        .with_note(
                            "The `@extends` tag provides concrete types for type parameters from a direct parent class.",
                        )
                        .with_help("For example: `@extends Box<string>`."),
                );

                continue;
            };

            let lowercase_parent_name = ascii_lowercase_word(parent_name.as_bytes());

            let has_parent = if class_like_metadata.kind.is_interface() {
                class_like_metadata.all_parent_interfaces.contains(&lowercase_parent_name)
            } else {
                class_like_metadata.all_parent_classes.contains(&lowercase_parent_name)
            };

            if !has_parent {
                class_like_metadata.issues.push(
                    Issue::error("`@extends` tag must refer to a direct parent class or interface.")
                        .with_code(ScanningIssueKind::InvalidExtendsTag)
                        .with_annotation(Annotation::primary(extended_type.r#type.span()).with_message(format!(
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

        for tag in document.tags() {
            let TagValue::Implements(implemented_type) = &tag.value else {
                continue;
            };

            let implemented_union = match builder::get_union_from_type(
                implemented_type.r#type,
                scope,
                &type_context,
                Some(original_name),
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
                            Annotation::primary(implemented_type.r#type.span()).with_message("Union types are not supported here."),
                        )
                        .with_note("The `@implements` tag provides concrete types for generics from a direct parent interface.")
                        .with_help("Provide a single parent interface, e.g., `@implements Serializable<string>`."),
                );

                continue;
            }

            let (parent_name, parent_parameters) = match implemented_union.get_single_owned() {
                TAtomic::Reference(TReference::Symbol { name, parameters, intersection_types: None, .. }) => {
                    (name, parameters)
                }
                atomic => {
                    let atomic_str = atomic.get_id();

                    class_like_metadata.issues.push(
                        Issue::error("The `@implements` tag expects a single interface type.")
                            .with_code(ScanningIssueKind::InvalidImplementsTag)
                            .with_annotation(
                                Annotation::primary(implemented_type.r#type.span())
                                    .with_message(format!("This must be an interface, not `{atomic_str}`.")),
                            )
                            .with_note("The `@implements` tag provides concrete types for type parameters from a direct parent interface.")
                            .with_help("Provide the single, interface name that this class implements."),
                    );

                    continue;
                }
            };

            let lowercase_parent_name = ascii_lowercase_word(parent_name.as_bytes());

            if !class_like_metadata.all_parent_interfaces.contains(&lowercase_parent_name) {
                class_like_metadata.issues.push(
                    Issue::error("The `@implements` tag must refer to a direct parent interface.")
                        .with_code(ScanningIssueKind::InvalidImplementsTag)
                        .with_annotation(Annotation::primary(implemented_type.r#type.span()).with_message(format!(
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

        for tag in document.tags() {
            let TagValue::RequireExtends(require_extend) = &tag.value else {
                continue;
            };

            let required_union =
                match builder::get_union_from_type(require_extend.r#type, scope, &type_context, Some(original_name)) {
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
                            Annotation::primary(require_extend.r#type.span())
                                .with_message("Union types are not supported here."),
                        )
                        .with_note("The `@require-extends` tag forces any type that inherits from this one to also extend a specific base class.")
                        .with_help("A class can only extend one other class. Provide a single parent class name."),
                );

                continue;
            }

            let (required_name, required_params) = if let TAtomic::Reference(TReference::Symbol {
                name,
                parameters,
                intersection_types,
                ..
            }) = required_union.get_single_owned()
            {
                if intersection_types.is_some() {
                    class_like_metadata.issues.push(
                        Issue::error("The `@require-extends` tag expects a single class name.")
                            .with_code(ScanningIssueKind::InvalidRequireExtendsTag)
                            .with_annotation(
                                Annotation::primary(require_extend.r#type.span())
                                    .with_message("Intersection types are not supported here."),
                            )
                            .with_note("The `@require-extends` tag forces any type that inherits from this one to also extend a specific base class.")
                            .with_help("A class can only extend one other class. Provide a single parent class name."),
                    );

                    continue;
                }

                (name, parameters)
            } else {
                class_like_metadata.issues.push(
                    Issue::error("The `@require-extends` tag expects a single class name.")
                        .with_code(ScanningIssueKind::InvalidRequireExtendsTag)
                        .with_annotation(
                            Annotation::primary(require_extend.r#type.span())
                                .with_message("This must be a class name, not a primitive or other complex type.")
                        )
                        .with_note("The `@require-extends` tag forces any type that inherits from this one to also extend a specific base class.")
                        .with_help("Provide the single, class name that all inheriting classes must extend."),
                );

                continue;
            };

            class_like_metadata.require_extends.insert(ascii_lowercase_word(required_name.as_bytes()));
            if let Some(required_params) = required_params {
                class_like_metadata.add_template_extended_offset(required_name, required_params);
            }
        }

        for tag in document.tags() {
            let TagValue::RequireImplements(require_implements) = &tag.value else {
                continue;
            };

            let required_union = match builder::get_union_from_type(
                require_implements.r#type,
                scope,
                &type_context,
                Some(original_name),
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
                            Annotation::primary(require_implements.r#type.span())
                                .with_message("Union types are not supported here."),
                        )
                        .with_note("The `@require-implements` tag forces any type that inherits from this one to also implement a specific interface.")
                        .with_help("To require that inheriting types implement multiple interfaces, use a separate `@require-implements` tag for each one."),
                );

                continue;
            }

            let (required_name, required_parameters) = if let TAtomic::Reference(TReference::Symbol {
                name,
                parameters,
                intersection_types,
                ..
            }) = required_union.get_single_owned()
            {
                if intersection_types.is_some() {
                    class_like_metadata.issues.push(
                        Issue::error("The `@require-implements` tag expects a single interface name.")
                            .with_code(ScanningIssueKind::InvalidRequireImplementsTag)
                            .with_annotation(
                                Annotation::primary(require_implements.r#type.span())
                                    .with_message("Intersection types are not supported here."),
                            )
                            .with_note("The `@require-implements` tag forces any type that inherits from this one to also implement a specific interface.")
                            .with_help("To require that inheriting types implement multiple interfaces, use a separate `@require-implements` tag for each one."),
                    );

                    continue;
                }

                (name, parameters)
            } else {
                class_like_metadata.issues.push(
                    Issue::error("The `@require-implements` tag expects a single interface name.")
                        .with_code(ScanningIssueKind::InvalidRequireImplementsTag)
                        .with_annotation(
                            Annotation::primary(require_implements.r#type.span())
                                .with_message("This must be an interface, not a primitive or other complex type."),
                        )
                        .with_note("The `@require-implements` tag forces any type that inherits from this one to also implement a specific interface.")
                        .with_help("Provide the single, interface name that all inheriting classes must implement."),
                );

                continue;
            };

            class_like_metadata.require_implements.insert(ascii_lowercase_word(required_name.as_bytes()));
            if let Some(required_parameters) = required_parameters {
                class_like_metadata.add_template_extended_offset(required_name, required_parameters);
            }
        }

        let mut inheritors = None;
        for tag in document.tags() {
            if let TagValue::Inheritors(inheritors_tag) = &tag.value {
                inheritors = Some(*inheritors_tag);
            }
        }

        if let Some(inheritors) = inheritors {
            match builder::get_union_from_type(inheritors.r#type, scope, &type_context, Some(original_name)) {
                Ok(inheritors_union) => {
                    for inheritor in inheritors_union.types.as_ref() {
                        match inheritor {
                            TAtomic::Reference(TReference::Symbol { name, intersection_types: None, .. }) => {
                                class_like_metadata
                                    .permitted_inheritors
                                    .get_or_insert_default()
                                    .insert(ascii_lowercase_word(name.as_bytes()));
                            }
                            _ => {
                                class_like_metadata.issues.push(
                                    Issue::error("The `@inheritors` tag only accepts class, interface, or enum names.")
                                        .with_code(ScanningIssueKind::InvalidInheritorsTag)
                                        .with_annotation(
                                            Annotation::primary(inheritors.r#type.span())
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
            }
        }

        for tag in document.tags() {
            let TagValue::Mixin(mixin) = &tag.value else {
                continue;
            };

            match builder::get_union_from_type(mixin.r#type, scope, &type_context, Some(original_name)) {
                Ok(mixin_type) => {
                    class_like_metadata.mixins.push(mixin_type);
                }
                Err(typing_error) => {
                    class_like_metadata.issues.push(
                        Issue::error("Could not resolve the type in the `@mixin` tag.")
                            .with_code(ScanningIssueKind::InvalidMixinTag)
                            .with_annotation(
                                Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                            )
                            .with_note(typing_error.note())
                            .with_help(typing_error.help()),
                    );
                }
            }
        }

        for tag in document.tags() {
            let TagValue::Method(method_tag) = &tag.value else {
                continue;
            };

            let method_name = ascii_lowercase_word(method_tag.name.value);
            class_like_metadata.methods.insert(method_name);
            class_like_metadata.pseudo_methods.insert(method_name);
            class_like_metadata
                .inheritable_method_ids
                .insert(method_name, MethodIdentifier::new(class_like_metadata.name, method_name));

            let method_id = (name, method_name);

            let mut function_like_metadata = FunctionLikeMetadata::new(
                FunctionLikeKind::Method,
                method_name,
                word(method_tag.name.value),
                method_tag.span(),
                MetadataFlags::empty(),
            );

            let Some(method_metadata) = function_like_metadata.method_metadata.as_mut() else {
                continue;
            };

            method_metadata.is_static = method_tag.is_static();
            method_metadata.visibility = match method_tag.visibility {
                None | Some(DocblockVisibility::Public(_)) => Visibility::Public,
                Some(DocblockVisibility::Protected(_)) => Visibility::Protected,
                Some(DocblockVisibility::Private(_)) => Visibility::Private,
            };

            function_like_metadata.flags.set(MetadataFlags::STATIC, method_tag.is_static());
            function_like_metadata.flags.set(MetadataFlags::MAGIC_METHOD, true);

            for argument in method_tag.parameters.entries.iter() {
                let mut function_parameter_metadata = FunctionLikeParameterMetadata::new(
                    VariableIdentifier(word(argument.parameter.value)),
                    argument.span(),
                    argument.parameter.span(),
                    MetadataFlags::empty(),
                );

                if let Some(parameter_type) = argument.r#type {
                    function_parameter_metadata.set_type_declaration_metadata(
                        get_type_metadata_from_type(parameter_type, Some(original_name), &type_context, scope).ok(),
                    );
                }

                if argument.is_variadic() {
                    function_parameter_metadata.flags.set(MetadataFlags::VARIADIC, true);
                }

                if argument.is_by_reference() {
                    function_parameter_metadata.flags.set(MetadataFlags::BY_REFERENCE, true);
                }

                if argument.is_optional() {
                    function_parameter_metadata.flags.set(MetadataFlags::HAS_DEFAULT, true);
                }

                function_like_metadata.parameters.push(function_parameter_metadata);
            }

            function_like_metadata.return_type_metadata = method_tag.return_type.and_then(|return_type| {
                get_type_metadata_from_type(return_type, Some(original_name), &type_context, scope).ok()
            });

            codebase.function_likes.insert(method_id, function_like_metadata);
        }

        for tag in document.tags() {
            let (property, is_read, is_write) = match &tag.value {
                TagValue::Property(property) => (property, true, true),
                TagValue::PropertyRead(property) => (property, true, false),
                TagValue::PropertyWrite(property) => (property, false, true),
                _ => continue,
            };

            let property_name = word(property.variable.value);
            let type_metadata = if let Some(property_type) = property.r#type {
                match get_type_metadata_from_type(property_type, Some(original_name), &type_context, scope) {
                    Ok(type_metadata) => Some(type_metadata),
                    Err(typing_error) => {
                        class_like_metadata.issues.push(
                            Issue::error("Could not resolve the property type in the `@property` tag.")
                                .with_code(ScanningIssueKind::InvalidPropertyTag)
                                .with_annotation(
                                    Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                                )
                                .with_note(typing_error.note())
                                .with_help(typing_error.help()),
                        );

                        None
                    }
                }
            } else {
                None
            };

            // `@property-read` and `@property-write` tags for the same property combine into
            // one readable and writable declaration; the later tag must not replace the earlier.
            if let Some(existing_property) = class_like_metadata.magic_properties.get_mut(&property_name) {
                let existing_is_write_only = existing_property.flags.is_writeonly();
                let existing_is_read_only = existing_property.flags.is_readonly();

                if is_read {
                    existing_property.read_visibility = Visibility::Public;
                    existing_property.flags.set(MetadataFlags::WRITEONLY, false);
                    if existing_is_write_only {
                        // The type slot holds the write-only tag's type; the read type takes the
                        // slot (it is what property accesses produce) and the write type moves
                        // into the dedicated write slot.
                        if existing_property.write_type_metadata.is_none() {
                            existing_property.write_type_metadata = existing_property.type_metadata.take();
                        }

                        existing_property.type_metadata = type_metadata.clone();
                    } else if existing_property.type_metadata.is_none() {
                        existing_property.type_metadata = type_metadata.clone();
                    }
                }

                if is_write {
                    existing_property.write_visibility = Visibility::Public;
                    existing_property.flags.set(MetadataFlags::READONLY, false);
                    if existing_is_read_only {
                        if existing_property.write_type_metadata.is_none() {
                            existing_property.write_type_metadata = type_metadata;
                        }
                    } else if existing_property.type_metadata.is_none() {
                        existing_property.type_metadata = type_metadata;
                    }
                }

                continue;
            }

            let mut new_property = PropertyMetadata::new(VariableIdentifier(property_name), MetadataFlags::empty());
            if is_read {
                new_property.read_visibility = Visibility::Public;
            }

            if is_write {
                new_property.write_visibility = Visibility::Public;
            }

            if is_read && !is_write {
                new_property.flags.set(MetadataFlags::READONLY, true);
            } else if !is_read && is_write {
                new_property.flags.set(MetadataFlags::WRITEONLY, true);
            }

            if let Some(type_metadata) = type_metadata {
                new_property.type_metadata.replace(type_metadata);
            }

            class_like_metadata.magic_properties.insert(property_name, new_property);
            class_like_metadata.magic_property_ids.insert(property_name, class_like_metadata.name);
        }
    }

    for member in members {
        let ClassLikeMember::Constant(constant) = member else {
            continue;
        };

        for constant_metadata in scan_class_like_constants(
            &mut class_like_metadata,
            constant,
            Some(original_name),
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

    for member in members {
        let ClassLikeMember::EnumCase(enum_case) = member else {
            continue;
        };

        let case_metadata = scan_enum_case(name, enum_case, context, scope, &class_like_metadata.constants);
        if class_like_metadata.constants.contains_key(&case_metadata.name) {
            continue;
        }

        class_like_metadata.enum_cases.insert(case_metadata.name, case_metadata);
    }

    if class_like_metadata.kind.is_enum()
        && let Some(enum_name_span) = class_like_metadata.name_span
    {
        let mut name_types = vec![];
        let mut value_types = vec![];
        let backing_type = class_like_metadata.enum_type.clone();

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

        let name = word("$name");
        let flags = MetadataFlags::READONLY | MetadataFlags::HAS_DEFAULT;
        let mut property_metadata = PropertyMetadata::new(VariableIdentifier(name), flags);
        property_metadata.type_declaration_metadata = Some(TypeMetadata::new(get_string(), enum_name_span));
        property_metadata.type_metadata = Some(TypeMetadata::new(name_union, enum_name_span));

        class_like_metadata.add_property_metadata(property_metadata);

        if let Some(enum_backing_type) = backing_type {
            let value = word("$value");

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

    for member in members {
        match member {
            ClassLikeMember::TraitUse(trait_use) => {
                for trait_use in &trait_use.trait_names {
                    let trait_name = context.resolved_names.get(trait_use);

                    class_like_metadata.add_used_trait(ascii_lowercase_word(trait_name));
                }

                if let TraitUseSpecification::Concrete(specification) = &trait_use.specification {
                    for adaptation in &specification.adaptations {
                        match adaptation {
                            TraitUseAdaptation::Precedence(_) => {}
                            TraitUseAdaptation::Alias(adaptation) => {
                                let method_name = ascii_lowercase_word(match &adaptation.method_reference {
                                    TraitUseMethodReference::Identifier(local_identifier) => local_identifier.value,
                                    TraitUseMethodReference::Absolute(reference) => reference.method_name.value,
                                });

                                let method_alias =
                                    adaptation.alias.as_ref().map(|alias| ascii_lowercase_word(alias.value));

                                if let Some(alias) = method_alias {
                                    class_like_metadata.add_trait_alias(method_name, alias);
                                }

                                if let Some(modifier) = &adaptation.modifier {
                                    let visibility = match modifier {
                                        Modifier::Public(_) => Visibility::Public,
                                        Modifier::Protected(_) => Visibility::Protected,
                                        Modifier::Private(_) => Visibility::Private,
                                        Modifier::Final(_) => {
                                            if let Some(method_alias) = method_alias {
                                                class_like_metadata.trait_final_map.insert(method_alias);
                                            } else {
                                                class_like_metadata.trait_final_map.insert(method_name);
                                            }

                                            continue;
                                        }
                                        _ => {
                                            continue;
                                        }
                                    };

                                    if let Some(method_alias) = method_alias {
                                        class_like_metadata.add_trait_visibility(method_alias, visibility);
                                    } else {
                                        class_like_metadata.add_trait_visibility(method_name, visibility);
                                    }
                                }
                            }
                        }
                    }
                }

                let Some(document) = parse_docblock(context, trait_use) else {
                    continue;
                };

                for parse_error in document.errors {
                    class_like_metadata.issues.push(
                        Issue::error("Failed to parse trait use docblock comment.")
                            .with_code(ScanningIssueKind::MalformedDocblockComment)
                            .with_annotation(
                                Annotation::primary(parse_error.span()).with_message(parse_error.to_string()),
                            )
                            .with_note(parse_error.note())
                            .with_help(parse_error.help()),
                    );
                }

                for tag in document.tags() {
                    let TagValue::Use(template_use) = &tag.value else {
                        continue;
                    };

                    let template_use_type = match builder::get_union_from_type(
                        template_use.r#type,
                        scope,
                        &type_context,
                        Some(original_name),
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
                                    Annotation::primary(template_use.r#type.span())
                                        .with_message("Union types are not allowed here."),
                                )
                                .with_note("The `@use` tag provides concrete types for generics from a trait.")
                                .with_help("Provide a single trait type, e.g., `@use MyTrait<string>`."),
                        );

                        continue;
                    }

                    let TAtomic::Reference(TReference::Symbol {
                        name: trait_name,
                        parameters: trait_parameters,
                        intersection_types: None,
                        ..
                    }) = template_use_type.get_single_owned()
                    else {
                        class_like_metadata.issues.push(
                            Issue::error("The `@use` tag expects a single trait type.")
                                .with_code(ScanningIssueKind::InvalidUseTag)
                                .with_annotation(
                                    Annotation::primary(template_use.r#type.span()).with_message(
                                        "This must be a trait name, not a primitive or other complex type.",
                                    ),
                                )
                                .with_note("The `@use` tag provides concrete types for generics from a trait.")
                                .with_help("Provide the single trait type, e.g., `@use MyTrait<string>`."),
                        );

                        continue;
                    };

                    let lowercase_trait_name = ascii_lowercase_word(trait_name.as_bytes());
                    if !class_like_metadata.used_traits.contains(&lowercase_trait_name) {
                        class_like_metadata.issues.push(
                            Issue::error("The `@use` tag must refer to a trait that is used.")
                                .with_code(ScanningIssueKind::InvalidUseTag)
                                .with_annotation(
                                    Annotation::primary(template_use.r#type.span()).with_message(format!(
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
                                        Annotation::primary(template_use.r#type.span()).with_message(
                                            "This tag must provide type parameters for the trait.",
                                        ),
                                    )
                                    .with_note("The `@use` tag is used to provide type information for the trait that is used in this class.")
                                    .with_help("Provide type parameters, e.g., `@use MyTrait<string>`."),
                            );
                        }
                    }
                }

                for tag in document.tags() {
                    let TagValue::Implements(template_implements) = &tag.value else {
                        continue;
                    };

                    class_like_metadata.issues.push(
                        Issue::error("The `@implements` tag is not allowed in trait use.")
                            .with_code(ScanningIssueKind::InvalidUseTag)
                            .with_annotation(
                                Annotation::primary(template_implements.span())
                                    .with_message("Use `@use` for traits, not `@implements`."),
                            )
                            .with_note("The `@implements` tag is used for interface, not traits.")
                            .with_help("Use `@use` to provide type information for traits."),
                    );
                }

                for tag in document.tags() {
                    let TagValue::Extends(template_extends) = &tag.value else {
                        continue;
                    };

                    class_like_metadata.issues.push(
                        Issue::error("The `@extends` tag is not allowed in trait use.")
                            .with_code(ScanningIssueKind::InvalidUseTag)
                            .with_annotation(
                                Annotation::primary(template_extends.span())
                                    .with_message("Use `@use` for traits, not `@extends`."),
                            )
                            .with_note("The `@extends` tag is used for classes and interfaces, not traits.")
                            .with_help("Use `@use` to provide type information for traits."),
                    );
                }
            }
            ClassLikeMember::Property(property) => {
                let properties =
                    scan_properties(property, &mut class_like_metadata, name, &type_context, context, scope);

                for property_metadata in properties {
                    class_like_metadata.add_property_metadata(property_metadata);
                }
            }
            _ => {}
        }
    }

    if class_like_metadata.attributes.iter().any(|attr| attr.name.as_bytes().eq_ignore_ascii_case(b"Deprecated")) {
        class_like_metadata.flags |= MetadataFlags::DEPRECATED;
    }

    Some(class_like_metadata)
}

fn create_enum_methods(codebase: &mut CodebaseMetadata, class_like: &mut ClassLikeMetadata, span: Span) {
    let mut add_method =
        |name: &[u8], class_like_metadata: &mut ClassLikeMetadata, function_like_metadata: FunctionLikeMetadata| {
            let name = ascii_lowercase_word(name);
            let method_id = (class_like_metadata.name, name);

            let method_identifier = MethodIdentifier::new(class_like_metadata.name, name);

            class_like_metadata.methods.insert(name);
            class_like_metadata.add_declaring_method_id(name, method_identifier);
            class_like_metadata.inheritable_method_ids.insert(name, method_identifier);
            codebase.function_likes.insert(method_id, function_like_metadata);
        };

    let enum_name_word = class_like.name;
    let backing_type = class_like.enum_type.clone();

    if let Some(backing_type) = backing_type {
        let from_method = create_enum_from_method(enum_name_word.as_bytes(), span, backing_type.clone());
        add_method(b"from", class_like, from_method);

        let try_from_method = create_enum_try_from_method(enum_name_word.as_bytes(), span, backing_type);
        add_method(b"tryFrom", class_like, try_from_method);
    }

    let has_cases = !class_like.enum_cases.is_empty();
    let cases_method = create_enum_cases_method(enum_name_word.as_bytes(), span, has_cases);
    add_method(b"cases", class_like, cases_method);
}

fn create_enum_from_method(enum_name: &[u8], enum_method_span: Span, backing_type: TAtomic) -> FunctionLikeMetadata {
    FunctionLikeMetadata {
        kind: FunctionLikeKind::Method,
        span: enum_method_span,
        name: word("from"),
        original_name: word("from"),
        name_span: Some(enum_method_span),
        parameters: vec![FunctionLikeParameterMetadata {
            attributes: vec![],
            name: VariableIdentifier(word("$value")),
            type_declaration_metadata: Some(TypeMetadata::new(
                TUnion::from_vec(vec![backing_type.clone()]),
                enum_method_span,
            )),
            type_metadata: Some(TypeMetadata::new(TUnion::from_vec(vec![backing_type]), enum_method_span)),
            out_type: None,
            default_type: None,
            span: enum_method_span,
            name_span: enum_method_span,
            flags: MetadataFlags::empty(),
        }],
        return_type_declaration_metadata: Some(TypeMetadata::new(
            TUnion::from_vec(vec![TAtomic::Object(TObject::Enum(TEnum { name: word(enum_name), case: None }))]),
            enum_method_span,
        )),
        return_type_metadata: Some(TypeMetadata::new(
            TUnion::from_vec(vec![TAtomic::Object(TObject::Enum(TEnum { name: word(enum_name), case: None }))]),
            enum_method_span,
        )),
        template_types: TemplateTypes::default(),
        attributes: vec![],
        method_metadata: Some(MethodMetadata {
            is_final: true,
            is_abstract: false,
            is_static: true,
            is_constructor: false,
            visibility: Visibility::Public,
            where_constraints: WordMap::default(),
        }),
        type_resolution_context: None,
        thrown_types: vec![TypeMetadata::new(get_named_object(word("ValueError"), None), enum_method_span)],
        issues: Vec::default(),
        assertions: BTreeMap::default(),
        if_true_assertions: BTreeMap::default(),
        if_false_assertions: BTreeMap::default(),
        assertions_inferred: false,
        globals_accessed: WordSet::default(),
        has_docblock: false,
        flags: MetadataFlags::POPULATED,
        version_constraint: crate::metadata::version_constraint::VersionConstraint::unconstrained(),
    }
}

fn create_enum_try_from_method(
    enum_name: &[u8],
    enum_method_span: Span,
    backing_type: TAtomic,
) -> FunctionLikeMetadata {
    FunctionLikeMetadata {
        kind: FunctionLikeKind::Method,
        span: enum_method_span,
        name: word("tryFrom"),
        original_name: word("tryFrom"),
        name_span: Some(enum_method_span),
        parameters: vec![FunctionLikeParameterMetadata {
            attributes: vec![],
            name: VariableIdentifier(word("$value")),
            type_declaration_metadata: Some(TypeMetadata::new(
                TUnion::from_vec(vec![backing_type.clone()]),
                enum_method_span,
            )),
            type_metadata: Some(TypeMetadata::new(TUnion::from_vec(vec![backing_type]), enum_method_span)),
            out_type: None,
            default_type: None,
            span: enum_method_span,
            name_span: enum_method_span,
            flags: MetadataFlags::empty(),
        }],
        return_type_declaration_metadata: Some(TypeMetadata::new(
            TUnion::from_vec(vec![
                TAtomic::Object(TObject::Enum(TEnum { name: word(enum_name), case: None })),
                TAtomic::Null,
            ]),
            enum_method_span,
        )),
        return_type_metadata: Some(TypeMetadata::new(
            TUnion::from_vec(vec![
                TAtomic::Object(TObject::Enum(TEnum { name: word(enum_name), case: None })),
                TAtomic::Null,
            ]),
            enum_method_span,
        )),
        template_types: TemplateTypes::default(),
        attributes: vec![],
        method_metadata: Some(MethodMetadata {
            is_final: true,
            is_abstract: false,
            is_static: true,
            is_constructor: false,
            visibility: Visibility::Public,
            where_constraints: WordMap::default(),
        }),
        type_resolution_context: None,
        thrown_types: vec![],
        issues: Vec::default(),
        assertions: BTreeMap::default(),
        if_true_assertions: BTreeMap::default(),
        if_false_assertions: BTreeMap::default(),
        assertions_inferred: false,
        globals_accessed: WordSet::default(),
        has_docblock: false,
        flags: MetadataFlags::POPULATED,
        version_constraint: crate::metadata::version_constraint::VersionConstraint::unconstrained(),
    }
}

fn create_enum_cases_method(enum_name: &[u8], enum_method_span: Span, has_cases: bool) -> FunctionLikeMetadata {
    FunctionLikeMetadata {
        kind: FunctionLikeKind::Method,
        span: enum_method_span,
        name: word("cases"),
        original_name: word("cases"),
        name_span: Some(enum_method_span),
        parameters: vec![],
        return_type_declaration_metadata: Some(TypeMetadata::new(
            if has_cases {
                get_non_empty_list(TUnion::from_vec(vec![TAtomic::Object(TObject::Enum(TEnum {
                    name: word(enum_name),
                    case: None,
                }))]))
            } else {
                get_list(TUnion::from_vec(vec![TAtomic::Object(TObject::Enum(TEnum {
                    name: word(enum_name),
                    case: None,
                }))]))
            },
            enum_method_span,
        )),
        return_type_metadata: Some(TypeMetadata::new(
            if has_cases {
                get_non_empty_list(TUnion::from_vec(vec![TAtomic::Object(TObject::Enum(TEnum {
                    name: word(enum_name),
                    case: None,
                }))]))
            } else {
                get_list(TUnion::from_vec(vec![TAtomic::Object(TObject::Enum(TEnum {
                    name: word(enum_name),
                    case: None,
                }))]))
            },
            enum_method_span,
        )),
        template_types: TemplateTypes::default(),
        attributes: vec![],
        method_metadata: Some(MethodMetadata {
            is_final: true,
            is_abstract: false,
            is_static: true,
            is_constructor: false,
            visibility: Visibility::Public,
            where_constraints: WordMap::default(),
        }),
        type_resolution_context: None,
        thrown_types: vec![],
        issues: Vec::default(),
        assertions: BTreeMap::default(),
        if_true_assertions: BTreeMap::default(),
        if_false_assertions: BTreeMap::default(),
        assertions_inferred: false,
        globals_accessed: WordSet::default(),
        has_docblock: false,
        flags: MetadataFlags::POPULATED,
        version_constraint: crate::metadata::version_constraint::VersionConstraint::unconstrained(),
    }
}
