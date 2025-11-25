use ahash::HashSet;
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
use crate::symbol::SymbolIdentifier;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::alias::TAlias;
use crate::ttype::atomic::populate_atomic_type;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::union::TUnion;
use crate::ttype::union::populate_union_type;

use super::methods::inherit_methods_from_parent;
use super::properties::inherit_properties_from_parent;
use super::templates::extend_template_parameters;

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

/// Recursively checks a TUnion for circular type alias references.
fn check_union_for_circular_refs(
    type_union: &TUnion,
    all_aliases: &mago_atom::AtomMap<TypeMetadata>,
    visiting: &mut AtomSet,
    path: &mut Vec<String>,
) -> Option<Vec<String>> {
    let nodes = type_union.get_all_child_nodes();
    for node in nodes {
        match node {
            TypeRef::Atomic(TAtomic::Reference(TReference::Symbol { name, .. })) => {
                if let Some(referenced_type) = all_aliases.get(name)
                    && let Some(cycle) =
                        detect_circular_type_reference(*name, referenced_type, all_aliases, visiting, path)
                {
                    return Some(cycle);
                }
            }
            _ => {
                // Other types are not relevant for circular reference detection
            }
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
/// Populates the metadata for a single class-like (class, interface, trait).
///
/// This function is potentially recursive, as it populates parent classes,
/// interfaces, and used traits before processing the current class-like.
/// It uses a remove/insert pattern to handle mutable borrowing across recursive calls.
///
/// NOTE: This is the old recursive version, kept for compatibility.
/// The new iterative version (populate_class_like_metadata_iterative) is preferred.
pub fn populate_class_like_metadata(
    classlike_name: &Atom,
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    safe_symbols: &AtomSet,
    safe_symbol_members: &HashSet<SymbolIdentifier>,
    population_stack: &mut AtomSet,
) {
    if let Some(metadata) = codebase.class_likes.get(classlike_name)
        && metadata.flags.is_populated()
    {
        return; // Already done, exit early
    }

    // Check if this class is currently being populated (circular dependency)
    if population_stack.contains(classlike_name) {
        return; // Exit early to avoid infinite recursion
    }

    let mut metadata = if let Some(metadata) = codebase.class_likes.remove(classlike_name) {
        metadata
    } else {
        return;
    };

    population_stack.insert(*classlike_name);
    for attribute_metadata in &metadata.attributes {
        symbol_references.add_symbol_reference_to_symbol(metadata.name, attribute_metadata.name, true);
    }

    for property_name in metadata.get_property_names() {
        metadata.add_declaring_property_id(property_name, *classlike_name);
    }

    for method_name in &metadata.methods {
        let method_id = MethodIdentifier::new(*classlike_name, *method_name);
        metadata.appearing_method_ids.insert(*method_name, method_id);
        metadata.declaring_method_ids.insert(*method_name, method_id);
    }

    for trait_name in metadata.used_traits.iter().copied().collect::<Vec<_>>() {
        populate_metadata_from_trait(
            &mut metadata,
            codebase,
            trait_name,
            symbol_references,
            safe_symbols,
            safe_symbol_members,
            population_stack,
        );
    }

    if let Some(parent_classname) = metadata.direct_parent_class {
        populate_metadata_from_parent_class_like(
            &mut metadata,
            codebase,
            parent_classname,
            symbol_references,
            safe_symbols,
            safe_symbol_members,
            population_stack,
        );
    }

    let direct_parent_interfaces = metadata.direct_parent_interfaces.iter().copied().collect::<Vec<_>>();
    for direct_parent_interface in direct_parent_interfaces {
        populate_interface_metadata_from_parent_interface(
            &mut metadata,
            codebase,
            direct_parent_interface,
            symbol_references,
            safe_symbols,
            safe_symbol_members,
            population_stack,
        );
    }

    for required_class in metadata.require_extends.iter().copied().collect::<Vec<_>>() {
        populate_metadata_from_required_class_like(
            &mut metadata,
            codebase,
            required_class,
            symbol_references,
            safe_symbols,
            safe_symbol_members,
            population_stack,
        );
    }

    for required_interface in metadata.require_implements.iter().copied().collect::<Vec<_>>() {
        populate_interface_metadata_from_parent_interface(
            &mut metadata,
            codebase,
            required_interface,
            symbol_references,
            safe_symbols,
            safe_symbol_members,
            population_stack,
        );
    }

    // Apply readonly to properties if the class is readonly
    if metadata.flags.is_readonly() {
        for property_metadata in metadata.properties.values_mut() {
            if !property_metadata.flags.is_static() {
                property_metadata.flags |= MetadataFlags::READONLY;
            }
        }
    }

    let pending_imports = std::mem::take(&mut metadata.imported_type_aliases);

    codebase.class_likes.insert(*classlike_name, metadata);
    for (local_name, (source_class_name, imported_type, import_span)) in pending_imports {
        populate_class_like_metadata(
            &source_class_name,
            codebase,
            symbol_references,
            safe_symbols,
            safe_symbol_members,
            population_stack,
        );

        if let Some(source_class) = codebase.class_likes.get(&source_class_name) {
            if source_class.type_aliases.contains_key(&imported_type) {
                let alias_metadata = TypeMetadata {
                    span: import_span,
                    type_union: TUnion::from_atomic(TAtomic::Alias(TAlias::new(source_class_name, imported_type))),
                    from_docblock: true,
                    inferred: false,
                };

                let metadata_mut = codebase.class_likes.get_mut(classlike_name).unwrap();
                metadata_mut.type_aliases.insert(local_name, alias_metadata);
            } else {
                let metadata_mut = codebase.class_likes.get_mut(classlike_name).unwrap();
                metadata_mut.issues.push(
                    Issue::error(format!("Type alias `{}` not found in class `{}`", imported_type, source_class_name))
                        .with_code("invalid-import-type")
                        .with_annotation(Annotation::primary(import_span))
                        .with_help(format!(
                            "Ensure that class `{}` defines a `@type {}` alias",
                            source_class_name, imported_type
                        )),
                );
            }
        } else {
            // Check if the class exists in the symbols registry
            // If it does, it just hasn't been populated yet, so skip validation for now
            // If it doesn't exist in symbols, it's truly not found
            if !codebase.symbols.contains(&source_class_name) {
                let metadata_mut = codebase
                    .class_likes
                    .get_mut(classlike_name)
                    .expect("Class-like metadata should exist in codebase after population of parents and traits");

                metadata_mut.issues.push(
                    Issue::error(format!("Class `{}` not found for type import", source_class_name))
                        .with_code("unknown-class-in-import-type")
                        .with_annotation(Annotation::primary(import_span))
                        .with_help(format!("Ensure that class `{}` is defined and scanned", source_class_name)),
                );
            }
        }
    }

    // Remove metadata from codebase to get exclusive access for the rest of the function
    let mut metadata = codebase
        .class_likes
        .remove(classlike_name)
        .expect("Class-like metadata should exist in codebase after population of parents and traits");

    // Check for circular type references in all type aliases
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

    // Add type aliases to method type resolution contexts
    // The actual resolution will happen lazily during analysis
    if !metadata.type_aliases.is_empty() {
        for method_name in &metadata.methods {
            let method_id = (*classlike_name, *method_name);
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

    // Remove from population stack now that we're done
    population_stack.remove(classlike_name);

    codebase.class_likes.insert(*classlike_name, metadata);
}

/// Populates interface data inherited from a parent interface.
fn populate_interface_metadata_from_parent_interface(
    metadata: &mut ClassLikeMetadata,
    codebase: &mut CodebaseMetadata,
    parent_interface: Atom,
    symbol_references: &mut SymbolReferences,
    safe_symbols: &AtomSet,
    safe_symbol_members: &HashSet<SymbolIdentifier>,
    population_stack: &mut AtomSet,
) {
    populate_class_like_metadata(
        &parent_interface,
        codebase,
        symbol_references,
        safe_symbols,
        safe_symbol_members,
        population_stack,
    );

    symbol_references.add_symbol_reference_to_symbol(metadata.name, parent_interface, true);

    let parent_interface_metadata = if let Some(parent_meta) = codebase.class_likes.get(&parent_interface) {
        parent_meta
    } else {
        metadata.invalid_dependencies.insert(parent_interface);
        return;
    };

    for (interface_constant_name, interface_constant_metadata) in &parent_interface_metadata.constants {
        if !metadata.constants.contains_key(interface_constant_name) {
            metadata.constants.insert(*interface_constant_name, interface_constant_metadata.clone());
        }
    }

    metadata.all_parent_interfaces.extend(parent_interface_metadata.all_parent_interfaces.iter().copied());
    metadata.invalid_dependencies.extend(parent_interface_metadata.invalid_dependencies.iter().copied());

    if let Some(inheritors) = &parent_interface_metadata.permitted_inheritors {
        metadata.permitted_inheritors.get_or_insert_default().extend(inheritors.iter().copied());
    }

    // Extend template parameters based on the parent interface's templates
    extend_template_parameters(metadata, parent_interface_metadata);
    // Inherit methods (appearing/declaring ids) from the parent interface
    // Pass codebase immutably if possible, or mutably if method inheritance logic needs it
    inherit_methods_from_parent(metadata, parent_interface_metadata, codebase);
    inherit_properties_from_parent(metadata, parent_interface_metadata);
}

/// Populates class-like data inherited from a parent class or trait.
fn populate_metadata_from_parent_class_like(
    metadata: &mut ClassLikeMetadata,
    codebase: &mut CodebaseMetadata,
    parent_class: Atom,
    symbol_references: &mut SymbolReferences,
    safe_symbols: &AtomSet,
    safe_symbol_members: &HashSet<SymbolIdentifier>,
    population_stack: &mut AtomSet,
) {
    populate_class_like_metadata(
        &parent_class,
        codebase,
        symbol_references,
        safe_symbols,
        safe_symbol_members,
        population_stack,
    );

    symbol_references.add_symbol_reference_to_symbol(metadata.name, parent_class, true);

    let parent_metadata = if let Some(parent_meta) = codebase.class_likes.get(&parent_class) {
        parent_meta
    } else {
        metadata.invalid_dependencies.insert(parent_class);
        return;
    };

    metadata.all_parent_classes.extend(parent_metadata.all_parent_classes.iter().copied());
    metadata.all_parent_interfaces.extend(parent_metadata.all_parent_interfaces.iter().copied());
    metadata.used_traits.extend(parent_metadata.used_traits.iter().copied());
    metadata.invalid_dependencies.extend(parent_metadata.invalid_dependencies.iter().copied());

    if let Some(inheritors) = &parent_metadata.permitted_inheritors {
        metadata.permitted_inheritors.get_or_insert_default().extend(inheritors.iter().copied());
    }

    extend_template_parameters(metadata, parent_metadata);

    inherit_methods_from_parent(metadata, parent_metadata, codebase);
    inherit_properties_from_parent(metadata, parent_metadata);

    for (parent_constant_name, parent_constant_metadata) in &parent_metadata.constants {
        if !metadata.constants.contains_key(parent_constant_name) {
            metadata.constants.insert(*parent_constant_name, parent_constant_metadata.clone());
        }
    }

    if parent_metadata.flags.has_consistent_templates() {
        metadata.flags |= MetadataFlags::CONSISTENT_TEMPLATES;
    }
}

/// Populates class-like data inherited from a parent class or trait.
fn populate_metadata_from_required_class_like(
    metadata: &mut ClassLikeMetadata,
    codebase: &mut CodebaseMetadata,
    parent_class: Atom,
    symbol_references: &mut SymbolReferences,
    safe_symbols: &AtomSet,
    safe_symbol_members: &HashSet<SymbolIdentifier>,
    population_stack: &mut AtomSet,
) {
    populate_class_like_metadata(
        &parent_class,
        codebase,
        symbol_references,
        safe_symbols,
        safe_symbol_members,
        population_stack,
    );

    symbol_references.add_symbol_reference_to_symbol(metadata.name, parent_class, true);

    let parent_metadata = if let Some(parent_meta) = codebase.class_likes.get(&parent_class) {
        parent_meta
    } else {
        metadata.invalid_dependencies.insert(parent_class);
        return;
    };

    metadata.require_extends.extend(parent_metadata.all_parent_classes.iter().copied());
    metadata.require_implements.extend(parent_metadata.all_parent_interfaces.iter().copied());
}

/// Populates class-like data inherited from a used trait.
fn populate_metadata_from_trait(
    metadata: &mut ClassLikeMetadata,
    codebase: &mut CodebaseMetadata,
    trait_name: Atom,
    symbol_references: &mut SymbolReferences,
    safe_symbols: &AtomSet,
    safe_symbol_members: &HashSet<SymbolIdentifier>,
    population_stack: &mut AtomSet,
) {
    populate_class_like_metadata(
        &trait_name,
        codebase,
        symbol_references,
        safe_symbols,
        safe_symbol_members,
        population_stack,
    );

    symbol_references.add_symbol_reference_to_symbol(metadata.name, trait_name, true);

    let Some(trait_metadata) = codebase.class_likes.get(&trait_name) else {
        metadata.invalid_dependencies.insert(trait_name);
        return;
    };

    // Inherit constants (if not already defined)
    for (trait_constant_name, trait_constant_metadata) in &trait_metadata.constants {
        // Always track that this constant came from this trait (used for override validation)
        metadata.trait_constant_ids.insert(*trait_constant_name, trait_name);

        if !metadata.constants.contains_key(trait_constant_name) {
            metadata.constants.insert(*trait_constant_name, trait_constant_metadata.clone());
        }
    }

    // Inherit the trait's parent interfaces (direct parents of the trait become parents of the user)
    metadata.all_parent_interfaces.extend(trait_metadata.direct_parent_interfaces.iter().copied());

    // Also inherit invalid dependencies from the trait
    metadata.invalid_dependencies.extend(trait_metadata.invalid_dependencies.iter().copied());

    // Inherit nested trait usages from the trait
    metadata.add_used_traits(trait_metadata.used_traits.iter().copied());

    // Extend template parameters based on the trait's templates
    extend_template_parameters(metadata, trait_metadata);

    // Inherit methods and properties from the trait
    inherit_methods_from_parent(metadata, trait_metadata, codebase);
    inherit_properties_from_parent(metadata, trait_metadata);
}

/// Populates types for properties, constants, enum cases, and type aliases within a class-like.
pub fn populate_class_like_types(
    name: &Atom,
    metadata: &mut ClassLikeMetadata,
    codebase_symbols: &crate::symbol::Symbols,
    symbol_references: &mut SymbolReferences,
    force_repopulation: bool,
) {
    let class_like_reference_source = ReferenceSource::Symbol(true, *name);

    for (property_name, property_metadata) in &mut metadata.properties {
        let property_reference_source = ReferenceSource::ClassLikeMember(true, *name, *property_name);

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

    for (constant_name, constant) in &mut metadata.constants {
        let constant_reference_source = ReferenceSource::ClassLikeMember(true, *name, *constant_name);

        for attribute_metadata in &constant.attributes {
            symbol_references.add_class_member_reference_to_symbol(
                (*name, *constant_name),
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
        let enum_case_reference_source = ReferenceSource::ClassLikeMember(true, *name, *enum_case_name);

        for attribute_metadata in &enum_case.attributes {
            symbol_references.add_class_member_reference_to_symbol(
                (*name, *enum_case_name),
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
            Some(&ReferenceSource::Symbol(true, *name)),
            symbol_references,
            force_repopulation,
        );
    }
}
