use itertools::Itertools;

use mago_atom::Atom;
use mago_atom::AtomSet;
use mago_reporting::Annotation;
use mago_reporting::Issue;

use crate::identifier::method::MethodIdentifier;
use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::ttype::TypeMetadata;
use crate::reference::ReferenceSource;
use crate::reference::SymbolReferences;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::alias::TAlias;
use crate::ttype::atomic::populate_atomic_type;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::union::TUnion;
use crate::ttype::union::populate_union_type;

use super::merge::merge_interface_metadata_from_parent_interface;
use super::merge::merge_metadata_from_parent_class_like;
use super::merge::merge_metadata_from_required_class_like;
use super::merge::merge_metadata_from_trait;

/// Detects circular references in a type definition by walking its dependencies.
///
/// Returns `Some(chain)` if a cycle is detected, where chain is the path of type names forming the cycle.
/// Returns `None` if no cycle is found.
fn detect_circular_type_reference(
    type_name: Atom,
    type_metadata: &TypeMetadata,
    all_aliases: &mago_atom::AtomMap<TypeMetadata>,
    visiting: &mut AtomSet,
    path: &mut Vec<String>,
) -> Option<Vec<String>> {
    // If we're already visiting this type, we found a cycle
    if visiting.contains(&type_name) {
        let mut cycle_chain = path.clone();
        cycle_chain.push(type_name.to_string());
        return Some(cycle_chain);
    }

    // Mark as visiting
    visiting.insert(type_name);
    path.push(type_name.to_string());

    // Walk through the type union looking for type alias references
    if let Some(cycle) = check_union_for_circular_refs(&type_metadata.type_union, all_aliases, visiting, path) {
        return Some(cycle);
    }

    // Done visiting this type
    visiting.remove(&type_name);
    path.pop();
    None
}

/// Recursively checks a `TUnion` for circular type alias references.
fn check_union_for_circular_refs(
    type_union: &TUnion,
    all_aliases: &mago_atom::AtomMap<TypeMetadata>,
    visiting: &mut AtomSet,
    path: &mut Vec<String>,
) -> Option<Vec<String>> {
    let nodes = type_union.get_all_child_nodes();
    for node in nodes {
        if let TypeRef::Atomic(TAtomic::Reference(TReference::Symbol { name, .. })) = node {
            if let Some(referenced_type) = all_aliases.get(name)
                && let Some(cycle) = detect_circular_type_reference(*name, referenced_type, all_aliases, visiting, path)
            {
                return Some(cycle);
            }
        } else {
            // Other types are not relevant for circular reference detection
        }
    }

    None
}

/// Populates the metadata for a single class-like iteratively (non-recursive).
///
/// This version assumes all dependencies have already been processed (via topological ordering).
/// It uses the remove/insert pattern to handle mutable borrowing, but no recursion.
///
/// # Safety
/// This function assumes the topological ordering guarantees that all dependencies
/// (parent classes, interfaces, traits) have already been populated.
/// Populates the metadata for a single class-like iteratively (non-recursive).
pub fn populate_class_like_metadata_iterative(
    classlike_name: Atom,
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
) {
    if let Some(metadata) = codebase.class_likes.get(&classlike_name)
        && metadata.flags.is_populated()
    {
        return; // Already done, exit early
    }

    let Some(mut metadata) = codebase.class_likes.remove(&classlike_name) else {
        return;
    };

    for attribute_metadata in &metadata.attributes {
        symbol_references.add_symbol_reference_to_symbol(metadata.name, attribute_metadata.name, true);
    }

    for property_name in metadata.get_property_names() {
        metadata.add_declaring_property_id(property_name, classlike_name);
    }

    for method_name in &metadata.methods {
        let method_id = MethodIdentifier::new(classlike_name, *method_name);
        metadata.appearing_method_ids.insert(*method_name, method_id);
        metadata.declaring_method_ids.insert(*method_name, method_id);
    }

    for trait_name in metadata.used_traits.iter().copied().sorted().collect::<Vec<_>>() {
        merge_metadata_from_trait(&mut metadata, codebase, trait_name, symbol_references);
    }

    if let Some(parent_classname) = metadata.direct_parent_class {
        merge_metadata_from_parent_class_like(&mut metadata, codebase, parent_classname, symbol_references);
    }

    let direct_parent_interfaces = metadata.direct_parent_interfaces.iter().copied().sorted().collect::<Vec<_>>();
    for direct_parent_interface in direct_parent_interfaces {
        merge_interface_metadata_from_parent_interface(
            &mut metadata,
            codebase,
            direct_parent_interface,
            symbol_references,
        );
    }

    for required_class in metadata.require_extends.iter().copied().sorted().collect::<Vec<_>>() {
        merge_metadata_from_required_class_like(&mut metadata, codebase, required_class, symbol_references);
    }

    for required_interface in metadata.require_implements.iter().copied().sorted().collect::<Vec<_>>() {
        merge_interface_metadata_from_parent_interface(&mut metadata, codebase, required_interface, symbol_references);
    }

    if metadata.flags.is_readonly() {
        for property_metadata in metadata.properties.values_mut() {
            if !property_metadata.flags.is_static() {
                property_metadata.flags |= MetadataFlags::READONLY;
            }
        }
    }

    let pending_imports = std::mem::take(&mut metadata.imported_type_aliases);
    for (local_name, (source_class_name, imported_type, import_span)) in pending_imports {
        if let Some(source_class) = codebase.class_likes.get(&source_class_name) {
            if source_class.type_aliases.contains_key(&imported_type) {
                let alias_metadata = TypeMetadata {
                    span: import_span,
                    type_union: TUnion::from_atomic(TAtomic::Alias(TAlias::new(source_class_name, imported_type))),
                    from_docblock: true,
                    inferred: false,
                };

                metadata.type_aliases.insert(local_name, alias_metadata);
            } else {
                metadata.issues.push(
                    Issue::error(format!("Type alias `{imported_type}` not found in class `{source_class_name}`"))
                        .with_code("invalid-import-type")
                        .with_annotation(Annotation::primary(import_span))
                        .with_help(format!(
                            "Ensure that class `{source_class_name}` defines a `@type {imported_type}` alias"
                        )),
                );
            }
        } else if !codebase.symbols.contains(&source_class_name) {
            metadata.issues.push(
                Issue::error(format!("Class `{source_class_name}` not found for type import"))
                    .with_code("unknown-class-in-import-type")
                    .with_annotation(Annotation::primary(import_span))
                    .with_help(format!("Ensure that class `{source_class_name}` is defined and scanned")),
            );
        }
    }

    for (type_name, type_metadata) in &metadata.type_aliases {
        let mut visiting = AtomSet::default();
        let mut path = Vec::new();

        if let Some(cycle) =
            detect_circular_type_reference(*type_name, type_metadata, &metadata.type_aliases, &mut visiting, &mut path)
        {
            metadata.issues.push(
                Issue::error(format!("Circular type reference detected: {}", cycle.join(" → ")))
                    .with_code(crate::issue::ScanningIssueKind::CircularTypeImport)
                    .with_annotation(
                        Annotation::primary(type_metadata.span)
                            .with_message("This type is part of a circular reference chain"),
                    )
                    .with_note(format!("The type reference chain creates a cycle: {}", cycle.join(" references ")))
                    .with_help("Reorganize your type definitions to avoid circular dependencies"),
            );
        }
    }

    if !metadata.type_aliases.is_empty() {
        for method_name in &metadata.methods {
            let method_id = (classlike_name, *method_name);
            if let Some(method_metadata) = codebase.function_likes.get_mut(&method_id) {
                let mut updated_context = method_metadata.type_resolution_context.clone().unwrap_or_default();
                for alias_name in metadata.type_aliases.keys() {
                    updated_context = updated_context.with_type_alias(*alias_name);
                }

                method_metadata.type_resolution_context = Some(updated_context);
            }
        }
    }

    metadata.mark_as_populated();
    codebase.class_likes.insert(classlike_name, metadata);
}

/// Populates types for properties, constants, enum cases, and type aliases within a class-like.
pub fn populate_class_like_types(
    name: Atom,
    metadata: &mut ClassLikeMetadata,
    codebase_symbols: &crate::symbol::Symbols,
    symbol_references: &mut SymbolReferences,
    force_repopulation: bool,
) {
    let class_like_reference_source = ReferenceSource::Symbol(true, name);

    for (property_name, property_metadata) in &mut metadata.properties {
        let property_reference_source = ReferenceSource::ClassLikeMember(true, name, *property_name);

        if let Some(signature) = property_metadata.type_declaration_metadata.as_mut() {
            populate_union_type(
                &mut signature.type_union,
                codebase_symbols,
                Some(&property_reference_source),
                symbol_references,
                force_repopulation,
            );
        }

        if let Some(signature) = property_metadata.type_metadata.as_mut() {
            populate_union_type(
                &mut signature.type_union,
                codebase_symbols,
                Some(&property_reference_source),
                symbol_references,
                force_repopulation,
            );
        }

        if let Some(default) = property_metadata.default_type_metadata.as_mut() {
            populate_union_type(
                &mut default.type_union,
                codebase_symbols,
                Some(&property_reference_source),
                symbol_references,
                force_repopulation,
            );
        }

        for hook in property_metadata.hooks.values_mut() {
            if let Some(hook_return_type) = &mut hook.return_type_metadata {
                populate_union_type(
                    &mut hook_return_type.type_union,
                    codebase_symbols,
                    Some(&property_reference_source),
                    symbol_references,
                    force_repopulation,
                );
            }

            let Some(hook_parameter) = hook.parameter.as_mut() else {
                continue;
            };

            if let Some(hook_parameter_type_declaration) = &mut hook_parameter.type_declaration_metadata {
                populate_union_type(
                    &mut hook_parameter_type_declaration.type_union,
                    codebase_symbols,
                    Some(&property_reference_source),
                    symbol_references,
                    force_repopulation,
                );
            }

            if let Some(hook_parameter_type) = &mut hook_parameter.type_metadata {
                populate_union_type(
                    &mut hook_parameter_type.type_union,
                    codebase_symbols,
                    Some(&property_reference_source),
                    symbol_references,
                    force_repopulation,
                );
            }

            if let Some(hook_parameter_out_type) = &mut hook_parameter.out_type {
                populate_union_type(
                    &mut hook_parameter_out_type.type_union,
                    codebase_symbols,
                    Some(&property_reference_source),
                    symbol_references,
                    force_repopulation,
                );
            }

            if let Some(hook_parameter_default_type) = &mut hook_parameter.default_type {
                populate_union_type(
                    &mut hook_parameter_default_type.type_union,
                    codebase_symbols,
                    Some(&property_reference_source),
                    symbol_references,
                    force_repopulation,
                );
            }
        }
    }

    for v in metadata.template_types.iter_mut().flat_map(|m| m.1.iter_mut()).map(|template| &mut template.1) {
        if v.needs_population() || force_repopulation {
            populate_union_type(
                v,
                codebase_symbols,
                Some(&class_like_reference_source),
                symbol_references,
                force_repopulation,
            );
        }
    }

    for template in &mut metadata.template_extended_offsets.values_mut().flatten() {
        if template.needs_population() || force_repopulation {
            populate_union_type(
                template,
                codebase_symbols,
                Some(&class_like_reference_source),
                symbol_references,
                force_repopulation,
            );
        }
    }

    for p in metadata.template_extended_parameters.values_mut().flat_map(|m| m.values_mut()) {
        if p.needs_population() || force_repopulation {
            populate_union_type(
                p,
                codebase_symbols,
                Some(&class_like_reference_source),
                symbol_references,
                force_repopulation,
            );
        }
    }

    for type_alias in metadata.type_aliases.values_mut() {
        populate_union_type(
            &mut type_alias.type_union,
            codebase_symbols,
            Some(&class_like_reference_source),
            symbol_references,
            force_repopulation,
        );
    }

    for mixin_type in &mut metadata.mixins {
        if mixin_type.needs_population() || force_repopulation {
            populate_union_type(
                mixin_type,
                codebase_symbols,
                Some(&class_like_reference_source),
                symbol_references,
                force_repopulation,
            );
        }
    }

    for (constant_name, constant) in &mut metadata.constants {
        let constant_reference_source = ReferenceSource::ClassLikeMember(true, name, *constant_name);

        for attribute_metadata in &constant.attributes {
            symbol_references.add_class_member_reference_to_symbol(
                (name, *constant_name),
                attribute_metadata.name,
                true,
            );
        }

        if let Some(signature) = &mut constant.type_metadata {
            populate_union_type(
                &mut signature.type_union,
                codebase_symbols,
                Some(&constant_reference_source),
                symbol_references,
                force_repopulation,
            );
        }

        if let Some(inferred) = &mut constant.inferred_type {
            populate_atomic_type(
                inferred,
                codebase_symbols,
                Some(&constant_reference_source),
                symbol_references,
                force_repopulation,
            );
        }
    }

    for (enum_case_name, enum_case) in &mut metadata.enum_cases {
        let enum_case_reference_source = ReferenceSource::ClassLikeMember(true, name, *enum_case_name);

        for attribute_metadata in &enum_case.attributes {
            symbol_references.add_class_member_reference_to_symbol(
                (name, *enum_case_name),
                attribute_metadata.name,
                true,
            );
        }

        if let Some(value_type) = &mut enum_case.value_type {
            populate_atomic_type(
                value_type,
                codebase_symbols,
                Some(&enum_case_reference_source),
                symbol_references,
                force_repopulation,
            );
        }
    }

    if let Some(enum_type) = &mut metadata.enum_type {
        populate_atomic_type(
            enum_type,
            codebase_symbols,
            Some(&ReferenceSource::Symbol(true, name)),
            symbol_references,
            force_repopulation,
        );
    }
}
