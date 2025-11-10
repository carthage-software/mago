use ahash::HashMap;
use ahash::HashSet;
use ahash::RandomState;
use indexmap::IndexMap;

use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::AtomSet;
use mago_atom::atom;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;

use crate::identifier::method::MethodIdentifier;
use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::GenericParent;
use crate::reference::ReferenceSource;
use crate::reference::SymbolReferences;
use crate::symbol::SymbolIdentifier;
use crate::symbol::Symbols;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::alias::TAlias;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::populate_atomic_type;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::template::TemplateResult;
use crate::ttype::template::inferred_type_replacer;
use crate::ttype::union::TUnion;
use crate::ttype::union::populate_union_type;

/// Populates the codebase metadata, resolving types and inheritance.
///
/// This function processes class-likes, function-likes, and constants to:
///
/// - Resolve type signatures (populating TUnion and TAtomic types).
/// - Calculate inheritance hierarchies (parent classes, interfaces, traits).
/// - Determine method and property origins (declaring vs. appearing).
/// - Build descendant maps for efficient lookup.
///
/// TODO(azjezz): This function is a performance bottleneck.
pub fn populate_codebase(
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    safe_symbols: AtomSet,
    safe_symbol_members: HashSet<SymbolIdentifier>,
) {
    let mut class_likes_to_repopulate = Vec::new();
    for (name, metadata) in codebase.class_likes.iter() {
        // Repopulate if not populated OR if user-defined and not marked safe.
        if !metadata.flags.is_populated() || (metadata.flags.is_user_defined() && !safe_symbols.contains(name)) {
            class_likes_to_repopulate.push(*name);
        }
    }

    for class_like_name in &class_likes_to_repopulate {
        if let Some(classlike_info) = codebase.class_likes.get_mut(class_like_name) {
            classlike_info.flags &= !MetadataFlags::POPULATED;
            classlike_info.declaring_property_ids.clear();
            classlike_info.appearing_property_ids.clear();
            classlike_info.declaring_method_ids.clear();
            classlike_info.appearing_method_ids.clear();
        }
    }

    for class_name in &class_likes_to_repopulate {
        let mut population_stack = AtomSet::default();
        populate_class_like_metadata(class_name, codebase, symbol_references, &safe_symbols, &mut population_stack);
    }

    for (name, function_like_metadata) in codebase.function_likes.iter_mut() {
        let force_repopulation = function_like_metadata.flags.is_user_defined() && !safe_symbols.contains(&name.0);

        let reference_source = if name.1.is_empty() || function_like_metadata.get_kind().is_closure() {
            // Top-level function or closure
            ReferenceSource::Symbol(true, name.0)
        } else {
            // Class method
            ReferenceSource::ClassLikeMember(true, name.0, name.1)
        };

        populate_function_like_metadata(
            function_like_metadata,
            &codebase.symbols,
            &reference_source,
            symbol_references,
            force_repopulation,
        );
    }

    for (name, metadata) in codebase.class_likes.iter_mut() {
        let userland_force_repopulation = metadata.flags.is_user_defined() && !safe_symbols.contains(name);
        let class_like_reference_source = ReferenceSource::Symbol(true, *name);

        for (property_name, property_metadata) in &mut metadata.properties {
            let property_reference_source = ReferenceSource::ClassLikeMember(true, *name, *property_name);

            if let Some(signature) = property_metadata.type_declaration_metadata.as_mut() {
                populate_union_type(
                    &mut signature.type_union,
                    &codebase.symbols,
                    Some(&property_reference_source),
                    symbol_references,
                    userland_force_repopulation,
                );
            }

            if let Some(signature) = property_metadata.type_metadata.as_mut() {
                populate_union_type(
                    &mut signature.type_union,
                    &codebase.symbols,
                    Some(&property_reference_source),
                    symbol_references,
                    userland_force_repopulation,
                );
            }

            if let Some(default) = property_metadata.default_type_metadata.as_mut() {
                populate_union_type(
                    &mut default.type_union,
                    &codebase.symbols,
                    Some(&property_reference_source),
                    symbol_references,
                    userland_force_repopulation,
                );
            }
        }

        for v in metadata.template_types.iter_mut().flat_map(|m| m.1.iter_mut()).map(|template| &mut template.1) {
            if v.needs_population() || userland_force_repopulation {
                populate_union_type(
                    v,
                    &codebase.symbols,
                    Some(&class_like_reference_source),
                    symbol_references,
                    userland_force_repopulation,
                );
            }
        }

        for template in &mut metadata.template_extended_offsets.values_mut().flatten() {
            if template.needs_population() || userland_force_repopulation {
                populate_union_type(
                    template,
                    &codebase.symbols,
                    Some(&class_like_reference_source),
                    symbol_references,
                    userland_force_repopulation,
                );
            }
        }

        for p in metadata.template_extended_parameters.values_mut().flat_map(|m| m.values_mut()) {
            if p.needs_population() || userland_force_repopulation {
                populate_union_type(
                    p,
                    &codebase.symbols,
                    Some(&class_like_reference_source),
                    symbol_references,
                    userland_force_repopulation,
                );
            }
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
                    &codebase.symbols,
                    Some(&constant_reference_source),
                    symbol_references,
                    userland_force_repopulation,
                );
            }

            if let Some(inferred) = &mut constant.inferred_type {
                populate_atomic_type(
                    inferred,
                    &codebase.symbols,
                    Some(&constant_reference_source),
                    symbol_references,
                    userland_force_repopulation,
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
                    &codebase.symbols,
                    Some(&enum_case_reference_source),
                    symbol_references,
                    userland_force_repopulation,
                );
            }
        }

        if let Some(enum_type) = &mut metadata.enum_type {
            populate_atomic_type(
                enum_type,
                &codebase.symbols,
                Some(&ReferenceSource::Symbol(true, *name)),
                symbol_references,
                userland_force_repopulation,
            );
        }
    }

    for (name, constant) in &mut codebase.constants {
        for attribute_metadata in &constant.attributes {
            symbol_references.add_symbol_reference_to_symbol(*name, attribute_metadata.name, true);
        }

        if let Some(type_metadata) = &mut constant.type_metadata {
            populate_union_type(
                &mut type_metadata.type_union,
                &codebase.symbols,
                Some(&ReferenceSource::Symbol(true, *name)),
                symbol_references,
                !safe_symbols.contains(name),
            );
        }

        if let Some(inferred_type) = &mut constant.inferred_type {
            populate_union_type(
                inferred_type,
                &codebase.symbols,
                Some(&ReferenceSource::Symbol(true, *name)),
                symbol_references,
                !safe_symbols.contains(name),
            );
        }
    }

    // Perform docblock inheritance for methods with @inheritDoc or no docblock
    inherit_method_docblocks(codebase);

    let mut direct_classlike_descendants = AtomMap::default();
    let mut all_classlike_descendants = AtomMap::default();

    for (class_like_name, class_like_metadata) in &codebase.class_likes {
        for parent_interface in &class_like_metadata.all_parent_interfaces {
            all_classlike_descendants
                .entry(*parent_interface)
                .or_insert_with(AtomSet::default)
                .insert(*class_like_name);
        }

        for parent_interface in &class_like_metadata.direct_parent_interfaces {
            direct_classlike_descendants
                .entry(*parent_interface)
                .or_insert_with(AtomSet::default)
                .insert(*class_like_name);
        }

        for parent_class in &class_like_metadata.all_parent_classes {
            all_classlike_descendants.entry(*parent_class).or_insert_with(AtomSet::default).insert(*class_like_name);
        }

        for used_trait in &class_like_metadata.used_traits {
            all_classlike_descendants.entry(*used_trait).or_default().insert(*class_like_name);
        }

        if let Some(parent_class) = &class_like_metadata.direct_parent_class {
            direct_classlike_descendants.entry(*parent_class).or_insert_with(AtomSet::default).insert(*class_like_name);
        }
    }

    codebase.all_class_like_descendants = all_classlike_descendants;
    codebase.direct_classlike_descendants = direct_classlike_descendants;
    codebase.safe_symbols = safe_symbols;
    codebase.safe_symbol_members = safe_symbol_members;
}

/// Populates metadata for a single function or method.
///
/// Resolves types for return types, parameters, template parameters, etc.
/// Adds symbol references based on attributes and types used.
fn populate_function_like_metadata(
    metadata: &mut FunctionLikeMetadata,
    codebase_symbols: &Symbols,
    reference_source: &ReferenceSource,
    symbol_references: &mut SymbolReferences,
    force_type_population: bool,
) {
    // Early exit if already populated and not forced
    if metadata.flags.is_populated() && !force_type_population {
        return;
    }

    for attribute_metadata in metadata.get_attributes() {
        match reference_source {
            ReferenceSource::Symbol(_, a) => {
                symbol_references.add_symbol_reference_to_symbol(*a, attribute_metadata.name, true)
            }
            ReferenceSource::ClassLikeMember(_, a, b) => {
                symbol_references.add_class_member_reference_to_symbol((*a, *b), attribute_metadata.name, true)
            }
        }
    }

    if let Some(return_type) = metadata.return_type_declaration_metadata.as_mut() {
        populate_union_type(
            &mut return_type.type_union,
            codebase_symbols,
            Some(reference_source),
            symbol_references,
            force_type_population,
        );
    }

    if let Some(return_type) = metadata.return_type_metadata.as_mut() {
        populate_union_type(
            &mut return_type.type_union,
            codebase_symbols,
            Some(reference_source),
            symbol_references,
            force_type_population,
        );
    }

    for parameter_metadata in metadata.get_parameters_mut() {
        if let Some(type_metadata) = parameter_metadata.type_metadata.as_mut() {
            populate_union_type(
                &mut type_metadata.type_union,
                codebase_symbols,
                Some(reference_source),
                symbol_references,
                force_type_population,
            );
        }

        if let Some(type_metadata) = parameter_metadata.out_type.as_mut() {
            populate_union_type(
                &mut type_metadata.type_union,
                codebase_symbols,
                Some(reference_source),
                symbol_references,
                force_type_population,
            );
        }

        if let Some(type_metadata) = parameter_metadata.default_type.as_mut() {
            populate_union_type(
                &mut type_metadata.type_union,
                codebase_symbols,
                Some(reference_source),
                symbol_references,
                force_type_population,
            );
        }

        for attribute_metadata in &parameter_metadata.attributes {
            match reference_source {
                ReferenceSource::Symbol(in_signature, a) => {
                    symbol_references.add_symbol_reference_to_symbol(*a, attribute_metadata.name, *in_signature)
                }
                ReferenceSource::ClassLikeMember(in_signature, a, b) => symbol_references
                    .add_class_member_reference_to_symbol((*a, *b), attribute_metadata.name, *in_signature),
            }
        }
    }

    for (_, type_parameter_map) in &mut metadata.template_types {
        for (_, type_parameter) in type_parameter_map {
            if force_type_population || type_parameter.needs_population() {
                populate_union_type(
                    type_parameter,
                    codebase_symbols,
                    Some(reference_source),
                    symbol_references,
                    force_type_population,
                );
            }
        }
    }

    if let Some(type_resolution_context) = metadata.type_resolution_context.as_mut() {
        for (_, type_parameter_map) in type_resolution_context.get_template_definitions_mut() {
            for (_, type_parameter) in type_parameter_map {
                if force_type_population || type_parameter.needs_population() {
                    populate_union_type(
                        type_parameter,
                        codebase_symbols,
                        Some(reference_source),
                        symbol_references,
                        force_type_population,
                    );
                }
            }
        }
    }

    if let Some(method_metadata) = metadata.method_metadata.as_mut() {
        for where_constraint in method_metadata.where_constraints.values_mut() {
            populate_union_type(
                &mut where_constraint.type_union,
                codebase_symbols,
                Some(reference_source),
                symbol_references,
                force_type_population,
            );
        }
    }

    for thrown_type in &mut metadata.thrown_types {
        populate_union_type(
            &mut thrown_type.type_union,
            codebase_symbols,
            Some(reference_source),
            symbol_references,
            force_type_population,
        );
    }

    for assertions in metadata.assertions.values_mut() {
        for assertion in assertions {
            if let Some(assertion_type) = assertion.get_type_mut() {
                populate_atomic_type(
                    assertion_type,
                    codebase_symbols,
                    Some(reference_source),
                    symbol_references,
                    force_type_population,
                );
            }
        }
    }

    for assertions in metadata.if_true_assertions.values_mut() {
        for assertion in assertions {
            if let Some(assertion_type) = assertion.get_type_mut() {
                populate_atomic_type(
                    assertion_type,
                    codebase_symbols,
                    Some(reference_source),
                    symbol_references,
                    force_type_population,
                );
            }
        }
    }

    for assertions in metadata.if_false_assertions.values_mut() {
        for assertion in assertions {
            if let Some(assertion_type) = assertion.get_type_mut() {
                populate_atomic_type(
                    assertion_type,
                    codebase_symbols,
                    Some(reference_source),
                    symbol_references,
                    force_type_population,
                );
            }
        }
    }

    metadata.flags |= MetadataFlags::POPULATED;
}

/// Detects circular references in a type definition by walking its dependencies.
///
/// Returns `Some(chain)` if a cycle is detected, where chain is the path of type names forming the cycle.
/// Returns `None` if no cycle is found.
fn detect_circular_type_reference(
    type_name: Atom,
    type_metadata: &TypeMetadata,
    all_aliases: &AtomMap<TypeMetadata>,
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
    all_aliases: &AtomMap<TypeMetadata>,
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

/// Populates the metadata for a single class-like (class, interface, trait).
///
/// This function is potentially recursive, as it populates parent classes,
/// interfaces, and used traits before processing the current class-like.
/// It uses a remove/insert pattern to handle mutable borrowing across recursive calls.
///
/// # Parameters
///
fn populate_class_like_metadata(
    classlike_name: &Atom,
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    safe_symbols: &AtomSet,
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
            population_stack,
        );
    }

    let direct_parent_interfaces = metadata.direct_parent_interfaces.clone();
    for direct_parent_interface in direct_parent_interfaces {
        populate_interface_metadata_from_parent_interface(
            &mut metadata,
            codebase,
            direct_parent_interface,
            symbol_references,
            safe_symbols,
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
        populate_class_like_metadata(&source_class_name, codebase, symbol_references, safe_symbols, population_stack);

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
    population_stack: &mut AtomSet,
) {
    populate_class_like_metadata(&parent_interface, codebase, symbol_references, safe_symbols, population_stack);

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
    population_stack: &mut AtomSet,
) {
    populate_class_like_metadata(&parent_class, codebase, symbol_references, safe_symbols, population_stack);

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
    population_stack: &mut AtomSet,
) {
    populate_class_like_metadata(&parent_class, codebase, symbol_references, safe_symbols, population_stack);

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
    population_stack: &mut AtomSet,
) {
    populate_class_like_metadata(&trait_name, codebase, symbol_references, safe_symbols, population_stack);

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

/// Inherits method declarations and appearances from a parent class-like.
/// Updates declaring_method_ids, appearing_method_ids, etc.
fn inherit_methods_from_parent(
    metadata: &mut ClassLikeMetadata,
    parent_metadata: &ClassLikeMetadata,
    codebase: &CodebaseMetadata,
) {
    let class_like_name = metadata.name;
    let parent_is_trait = parent_metadata.kind.is_trait();

    // Register where methods appear (can never be in a trait in the final appearing_method_ids)
    for (method_name_lc, appearing_method_id) in &parent_metadata.appearing_method_ids {
        // Build a list of aliased method names
        let mut aliased_method_names = vec![*method_name_lc];

        if parent_is_trait && !metadata.trait_alias_map.is_empty() {
            // Find all aliases for this method
            // trait_alias_map is: original_method_name -> alias_name
            aliased_method_names.extend(
                metadata
                    .get_trait_alias_map()
                    .iter()
                    .filter(|(original, _)| *original == method_name_lc)
                    .map(|(_, alias)| *alias),
            );
        }

        for aliased_method_name in aliased_method_names {
            if metadata.has_appearing_method(&aliased_method_name) {
                continue;
            }

            let implemented_method_id = MethodIdentifier::new(class_like_name, aliased_method_name);

            let final_appearing_id = if parent_is_trait { implemented_method_id } else { *appearing_method_id };

            metadata.appearing_method_ids.insert(aliased_method_name, final_appearing_id);

            let this_method_id_str = format!("{}::{}", class_like_name, method_name_lc);

            if codebase.function_likes.contains_key(&(class_like_name, aliased_method_name)) {
                let mut potential_ids = HashMap::default();
                potential_ids.insert(this_method_id_str, true);
                metadata.potential_declaring_method_ids.insert(aliased_method_name, potential_ids);
            } else {
                if let Some(parent_potential_method_ids) =
                    parent_metadata.get_potential_declaring_method_id(&aliased_method_name)
                {
                    metadata
                        .potential_declaring_method_ids
                        .insert(aliased_method_name, parent_potential_method_ids.clone());
                }

                metadata.add_potential_declaring_method(aliased_method_name, this_method_id_str);

                let parent_method_id_str = format!("{}::{}", parent_metadata.name, method_name_lc);
                metadata.add_potential_declaring_method(aliased_method_name, parent_method_id_str);
            }
        }
    }

    // Register where methods are declared
    for (method_name_lc, declaring_method_id) in &parent_metadata.inheritable_method_ids {
        if !method_name_lc.eq(&atom("__construct")) || parent_metadata.flags.has_consistent_constructor() {
            if parent_is_trait {
                let declaring_class = declaring_method_id.get_class_name();

                if codebase
                    .function_likes
                    .get(&(*declaring_class, *method_name_lc))
                    .and_then(|meta| meta.method_metadata.as_ref())
                    .is_some_and(|method| method.is_abstract)
                {
                    metadata.add_overridden_method_parent(*method_name_lc, *declaring_method_id);
                }
            } else {
                metadata.add_overridden_method_parent(*method_name_lc, *declaring_method_id);
            }

            if let Some(existing_overridden) = metadata.overridden_method_ids.get_mut(method_name_lc)
                && let Some(parent_overridden_map) = parent_metadata.overridden_method_ids.get(method_name_lc)
            {
                existing_overridden.extend(parent_overridden_map.iter().map(|(k, v)| (*k, *v)));
            }
        }

        let mut aliased_method_names = vec![*method_name_lc];

        if parent_is_trait && !metadata.trait_alias_map.is_empty() {
            // trait_alias_map is: original_method_name -> alias_name
            aliased_method_names.extend(
                metadata
                    .get_trait_alias_map()
                    .iter()
                    .filter(|(original, _)| *original == method_name_lc)
                    .map(|(_, alias)| *alias),
            );
        }

        for aliased_method_name in aliased_method_names {
            if let Some(implementing_method_id) = metadata.declaring_method_ids.get(&aliased_method_name) {
                let implementing_class = implementing_method_id.get_class_name();
                let implementing_method_name = implementing_method_id.get_method_name();

                if !codebase.method_is_abstract(implementing_class, implementing_method_name) {
                    continue;
                }
            }

            metadata.declaring_method_ids.insert(aliased_method_name, *declaring_method_id);
            metadata.inheritable_method_ids.insert(aliased_method_name, *declaring_method_id);
        }
    }
}

/// Inherits property declarations and appearances from a parent class-like.
/// Updates declaring_property_ids, appearing_property_ids, etc.
fn inherit_properties_from_parent(metadata: &mut ClassLikeMetadata, parent_metadata: &ClassLikeMetadata) {
    let classlike_name = metadata.name;
    let is_trait = metadata.kind.is_trait();
    let parent_is_trait = parent_metadata.kind.is_trait();

    for (property_name, appearing_classlike) in &parent_metadata.appearing_property_ids {
        if metadata.has_appearing_property(property_name) {
            continue;
        }

        if !parent_is_trait
            && let Some(parent_property_metadata) = parent_metadata.properties.get(property_name)
            && parent_property_metadata.is_final()
        {
            continue;
        }

        metadata
            .appearing_property_ids
            .insert(*property_name, if is_trait { classlike_name } else { *appearing_classlike });

        if parent_is_trait && let Some(parent_property) = parent_metadata.properties.get(property_name) {
            metadata.properties.entry(*property_name).or_insert_with(|| parent_property.clone());
        }
    }

    for (property_name, declaring_classlike) in &parent_metadata.declaring_property_ids {
        if metadata.declaring_property_ids.contains_key(property_name) {
            continue;
        }

        if !parent_is_trait
            && let Some(parent_property_metadata) = parent_metadata.properties.get(property_name)
            && parent_property_metadata.is_final()
        {
            continue;
        }

        metadata.declaring_property_ids.insert(*property_name, *declaring_classlike);
    }

    for (property_name, inheritable_classlike) in &parent_metadata.inheritable_property_ids {
        let mut is_overridable = true;
        if !parent_is_trait {
            if let Some(parent_property_metadata) = parent_metadata.properties.get(property_name)
                && parent_property_metadata.is_final()
            {
                is_overridable = false;
            }

            if is_overridable {
                metadata.overridden_property_ids.entry(*property_name).or_default().insert(*inheritable_classlike);
            }
        }

        if is_overridable {
            metadata.inheritable_property_ids.insert(*property_name, *inheritable_classlike);
        }
    }
}

/// Extends the template parameter map of `metadata` based on `parent_metadata`.
/// Handles resolving template types inherited from parents/traits.
fn extend_template_parameters(metadata: &mut ClassLikeMetadata, parent_metadata: &ClassLikeMetadata) {
    let parent_name = parent_metadata.name;

    if !parent_metadata.template_types.is_empty() {
        metadata.template_extended_parameters.entry(parent_name).or_default();

        if let Some(parent_offsets) = metadata.template_extended_offsets.get(&parent_name).cloned() {
            let parent_template_type_names = parent_metadata.get_template_type_names();

            for (i, extended_type_arc) in parent_offsets.into_iter().enumerate() {
                if let Some(mapped_name) = parent_template_type_names.get(i).copied() {
                    metadata.add_template_extended_parameter(parent_name, mapped_name, extended_type_arc);
                }
            }

            let current_child_extended_params = metadata.template_extended_parameters.clone();
            for (grandparent_fqcn, type_map) in &parent_metadata.template_extended_parameters {
                for (template_name, type_to_resolve_arc) in type_map {
                    let resolved_type = extend_type(type_to_resolve_arc, &current_child_extended_params);

                    metadata.add_template_extended_parameter(*grandparent_fqcn, *template_name, resolved_type);
                }
            }
        } else {
            for (parameter_name, parameter_type_map) in &parent_metadata.template_types {
                for (_, parameter_type) in parameter_type_map {
                    metadata.add_template_extended_parameter(parent_name, *parameter_name, parameter_type.clone());
                }
            }

            metadata.extend_template_extended_parameters(parent_metadata.template_extended_parameters.clone());
        }
    } else {
        // Inherit the parent's extended parameters map directly.
        metadata.extend_template_extended_parameters(parent_metadata.template_extended_parameters.clone());
    }
}

/// Resolves a TUnion that might contain generic parameters, using the provided
/// extended parameter map.
///
/// Example: If `extended_type` is `T` (generic param) and `template_extended_parameters`
/// maps `T` defined on `ParentClass` to `string`, this returns a `TUnion` containing `string`.
fn extend_type(
    extended_type: &TUnion,
    template_extended_parameters: &AtomMap<IndexMap<Atom, TUnion, RandomState>>,
) -> TUnion {
    if !extended_type.has_template() {
        return extended_type.clone();
    }

    let mut extended_types = Vec::new();

    let mut worklist = extended_type.types.clone().into_owned();
    while let Some(atomic_type) = worklist.pop() {
        if let TAtomic::GenericParameter(TGenericParameter {
            parameter_name,
            defining_entity: GenericParent::ClassLike(defining_entity),
            ..
        }) = &atomic_type
            && let Some(extended_parameters) = template_extended_parameters.get(defining_entity)
            && let Some(referenced_type) = extended_parameters.get(parameter_name)
        {
            extended_types.extend(referenced_type.types.clone().into_owned());
            continue;
        }

        extended_types.push(atomic_type);
    }

    TUnion::from_vec(extended_types)
}

/// Performs docblock inheritance for methods that need it.
///
/// Methods inherit docblock from their parent class/interface/trait if:
/// 1. They have an explicit `@inheritDoc` tag, OR
/// 2. They have NO docblock at all (implicit inheritance)
///
/// Template parameters (e.g., `T`) are substituted with concrete types
/// (e.g., `string` when class implements `Interface<string>`).
fn inherit_method_docblocks(codebase: &mut CodebaseMetadata) {
    let mut inheritance_work: Vec<(Atom, Atom, Atom, Atom)> = Vec::new();

    for (class_name, class_metadata) in &codebase.class_likes {
        for (method_name, method_ids) in &class_metadata.overridden_method_ids {
            let child_method_id = (*class_name, *method_name);

            let Some(child_method) = codebase.function_likes.get(&child_method_id) else {
                continue;
            };

            if !child_method.needs_docblock_inheritance() {
                continue;
            }

            let mut parent_method_id = None;

            if let Some(parent_class) = &class_metadata.direct_parent_class
                && method_ids.contains_key(parent_class)
            {
                parent_method_id = Some((*parent_class, *method_name));
            }

            if parent_method_id.is_none() {
                for interface in &class_metadata.all_parent_interfaces {
                    if method_ids.contains_key(interface) {
                        parent_method_id = Some((*interface, *method_name));
                        break;
                    }
                }
            }

            if parent_method_id.is_none() {
                for trait_name in &class_metadata.used_traits {
                    if method_ids.contains_key(trait_name) {
                        parent_method_id = Some((*trait_name, *method_name));
                        break;
                    }
                }
            }

            if let Some((parent_class, parent_method)) = parent_method_id {
                inheritance_work.push((*class_name, *method_name, parent_class, parent_method));
            }
        }
    }

    for (class_name, method_name, parent_class, parent_method) in inheritance_work {
        let child_method_id = (class_name, method_name);
        let parent_method_id = (parent_class, parent_method);

        let parent_method = match codebase.function_likes.get(&parent_method_id) {
            Some(m) => m,
            None => continue,
        };

        let parent_return_type = parent_method.return_type_metadata.clone();
        let parent_parameters = parent_method.parameters.clone();
        let parent_template_types = parent_method.template_types.clone();
        let parent_thrown_types = parent_method.thrown_types.clone();
        let parent_assertions = parent_method.assertions.clone();
        let parent_if_true_assertions = parent_method.if_true_assertions.clone();
        let parent_if_false_assertions = parent_method.if_false_assertions.clone();

        let child_class = match codebase.class_likes.get(&class_name) {
            Some(c) => c,
            None => continue,
        };

        let template_map = child_class.template_extended_parameters.get(&parent_class);

        let template_result = template_map.map(|template_map| {
            let mut template_result = TemplateResult::default();
            for (template_name, concrete_type) in template_map {
                template_result.add_lower_bound(
                    *template_name,
                    GenericParent::ClassLike(parent_class),
                    concrete_type.clone(),
                );
            }
            template_result
        });

        let substituted_return_type = if let Some(parent_return) = parent_return_type.as_ref() {
            let mut return_type = parent_return.type_union.clone();
            if let Some(ref template_result) = template_result {
                return_type = inferred_type_replacer::replace(&return_type, template_result, codebase);
            }
            Some((return_type, parent_return.span))
        } else {
            None
        };

        let substituted_param_types: Vec<Option<(TUnion, Span)>> = parent_parameters
            .iter()
            .map(|parent_param| {
                if let Some(parent_param_type) = parent_param.type_metadata.as_ref() {
                    let mut param_type = parent_param_type.type_union.clone();
                    if let Some(ref template_result) = template_result {
                        param_type = inferred_type_replacer::replace(&param_type, template_result, codebase);
                    }
                    Some((param_type, parent_param_type.span))
                } else {
                    None
                }
            })
            .collect();

        let substituted_thrown_types: Vec<TypeMetadata> = parent_thrown_types
            .iter()
            .map(|throw_type| {
                let mut throw_type_union = throw_type.type_union.clone();
                if let Some(ref template_result) = template_result {
                    throw_type_union = inferred_type_replacer::replace(&throw_type_union, template_result, codebase);
                }

                TypeMetadata {
                    type_union: throw_type_union,
                    span: throw_type.span,
                    from_docblock: true,
                    inferred: false,
                }
            })
            .collect();

        let child_method = match codebase.function_likes.get_mut(&child_method_id) {
            Some(m) => m,
            None => continue,
        };

        let should_inherit_return = child_method.return_type_metadata.is_none()
            || !child_method.return_type_metadata.as_ref().unwrap().from_docblock;

        if should_inherit_return && let Some((return_type, span)) = substituted_return_type {
            child_method.return_type_metadata =
                Some(TypeMetadata { type_union: return_type, span, from_docblock: true, inferred: false });
        }

        for (i, substituted_param) in substituted_param_types.iter().enumerate() {
            if let Some(child_param) = child_method.parameters.get_mut(i) {
                let should_inherit_param = child_param.type_metadata.is_none()
                    || child_param.type_metadata.as_ref().is_some_and(|m| !m.from_docblock);

                if should_inherit_param && let Some((param_type, span)) = substituted_param {
                    child_param.type_metadata = Some(TypeMetadata {
                        type_union: param_type.clone(),
                        span: *span,
                        from_docblock: true,
                        inferred: false,
                    });
                }
            }
        }

        if child_method.template_types.is_empty() && !parent_template_types.is_empty() {
            child_method.template_types = parent_template_types;
        }

        if child_method.thrown_types.is_empty() && !substituted_thrown_types.is_empty() {
            child_method.thrown_types = substituted_thrown_types;
        }

        if child_method.assertions.is_empty() && !parent_assertions.is_empty() {
            child_method.assertions = parent_assertions;
        }

        if child_method.if_true_assertions.is_empty() && !parent_if_true_assertions.is_empty() {
            child_method.if_true_assertions = parent_if_true_assertions;
        }

        if child_method.if_false_assertions.is_empty() && !parent_if_false_assertions.is_empty() {
            child_method.if_false_assertions = parent_if_false_assertions;
        }
    }
}
