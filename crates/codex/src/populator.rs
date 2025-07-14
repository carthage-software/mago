use ahash::HashMap;
use ahash::HashSet;
use ahash::RandomState;
use indexmap::IndexMap;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;

use crate::is_method_abstract;
use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::misc::GenericParent;
use crate::reference::ReferenceSource;
use crate::reference::SymbolReferences;
use crate::symbol::Symbols;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::populate_atomic_type;
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
    interner: &ThreadedInterner,
    symbol_references: &mut SymbolReferences,
    safe_symbols: HashSet<StringIdentifier>,
    safe_symbol_members: HashSet<(StringIdentifier, StringIdentifier)>,
) {
    let mut class_likes_to_repopulate = Vec::new();
    for (name, metadata) in codebase.class_likes.iter() {
        // Repopulate if not populated OR if user-defined and not marked safe.
        if !metadata.is_populated || (metadata.is_user_defined() && !safe_symbols.contains(name)) {
            class_likes_to_repopulate.push(*name);
        }
    }

    for class_like_name in &class_likes_to_repopulate {
        if let Some(classlike_info) = codebase.class_likes.get_mut(class_like_name) {
            classlike_info.is_populated = false;
            classlike_info.declaring_property_ids.clear();
            classlike_info.appearing_property_ids.clear();
            classlike_info.declaring_method_ids.clear();
            classlike_info.appearing_method_ids.clear();
        }
    }

    for class_name in &class_likes_to_repopulate {
        populate_class_like_metadata(class_name, codebase, interner, symbol_references, &safe_symbols);
    }

    for (name, function_like_metadata) in codebase.function_likes.iter_mut() {
        let force_repopulation = function_like_metadata.is_user_defined() && !safe_symbols.contains(&name.0);

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
            interner,
            &reference_source,
            symbol_references,
            force_repopulation,
        );
    }

    for (name, metadata) in codebase.class_likes.iter_mut() {
        let userland_force_repopulation = metadata.is_user_defined() && !safe_symbols.contains(name);
        let class_like_reference_source = ReferenceSource::Symbol(true, *name);

        for (property_name, property_metadata) in metadata.get_properties_mut() {
            let property_reference_source = ReferenceSource::ClassLikeMember(true, *name, *property_name);

            if let Some(signature) = property_metadata.type_declaration_metadata.as_mut() {
                populate_union_type(
                    &mut signature.type_union,
                    &codebase.symbols,
                    interner,
                    Some(&property_reference_source),
                    symbol_references,
                    userland_force_repopulation,
                );
            }

            if let Some(signature) = property_metadata.type_metadata.as_mut() {
                populate_union_type(
                    &mut signature.type_union,
                    &codebase.symbols,
                    interner,
                    Some(&property_reference_source),
                    symbol_references,
                    userland_force_repopulation,
                );
            }

            if let Some(default) = property_metadata.default_type_metadata.as_mut() {
                populate_union_type(
                    &mut default.type_union,
                    &codebase.symbols,
                    interner,
                    Some(&property_reference_source),
                    symbol_references,
                    userland_force_repopulation,
                );
            }
        }

        for map in metadata.template_extended_parameters.values_mut() {
            for (_, v) in map {
                if v.needs_population() || userland_force_repopulation {
                    populate_union_type(
                        v,
                        &codebase.symbols,
                        interner,
                        Some(&class_like_reference_source),
                        symbol_references,
                        userland_force_repopulation,
                    );
                }
            }
        }

        for (_, map) in metadata.get_template_types_mut() {
            for (_, v) in map {
                if v.needs_population() || userland_force_repopulation {
                    populate_union_type(
                        v,
                        &codebase.symbols,
                        interner,
                        Some(&class_like_reference_source),
                        symbol_references,
                        userland_force_repopulation,
                    );
                }
            }
        }

        for (constant_name, constant) in &mut metadata.constants {
            let constant_reference_source = ReferenceSource::ClassLikeMember(true, *name, *constant_name);

            for attribute_metadata in &constant.attributes {
                symbol_references.add_class_member_reference_to_symbol(
                    (*name, *constant_name),
                    attribute_metadata.get_name(),
                    true,
                );
            }

            if let Some(signature) = &mut constant.type_metadata {
                populate_union_type(
                    &mut signature.type_union,
                    &codebase.symbols,
                    interner,
                    Some(&constant_reference_source),
                    symbol_references,
                    userland_force_repopulation,
                );
            }

            if let Some(inferred) = &mut constant.inferred_type {
                populate_atomic_type(
                    inferred,
                    &codebase.symbols,
                    interner,
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
                    attribute_metadata.get_name(),
                    true,
                );
            }

            if let Some(value_type) = &mut enum_case.value_type {
                populate_atomic_type(
                    value_type,
                    &codebase.symbols,
                    interner,
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
                interner,
                Some(&ReferenceSource::Symbol(true, *name)),
                symbol_references,
                userland_force_repopulation,
            );
        }
    }

    for (name, constant) in &mut codebase.constants {
        for attribute_metadata in &constant.attributes {
            symbol_references.add_symbol_reference_to_symbol(*name, attribute_metadata.get_name(), true);
        }

        if let Some(inferred_type) = &mut constant.inferred_type {
            populate_union_type(
                inferred_type,
                &codebase.symbols,
                interner,
                Some(&ReferenceSource::Symbol(true, *name)),
                symbol_references,
                !safe_symbols.contains(name),
            );
        }
    }

    let mut direct_classlike_descendants = HashMap::default();
    let mut all_classlike_descendants = HashMap::default();

    for (class_like_name, class_like_metadata) in &codebase.class_likes {
        for parent_interface in class_like_metadata.get_all_parent_interfaces() {
            all_classlike_descendants
                .entry(*parent_interface)
                .or_insert_with(HashSet::default)
                .insert(*class_like_name);
        }

        for parent_interface in class_like_metadata.get_direct_parent_interfaces() {
            direct_classlike_descendants
                .entry(parent_interface)
                .or_insert_with(HashSet::default)
                .insert(*class_like_name);
        }

        for parent_class in class_like_metadata.get_all_parent_classes() {
            all_classlike_descendants.entry(*parent_class).or_insert_with(HashSet::default).insert(*class_like_name);
        }

        for used_trait in class_like_metadata.get_used_traits() {
            all_classlike_descendants.entry(used_trait).or_default().insert(*class_like_name);
        }

        if let Some(parent_class) = class_like_metadata.get_direct_parent_class() {
            direct_classlike_descendants.entry(parent_class).or_insert_with(HashSet::default).insert(*class_like_name);
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
    interner: &ThreadedInterner,
    reference_source: &ReferenceSource,
    symbol_references: &mut SymbolReferences,
    force_type_population: bool,
) {
    // Early exit if already populated and not forced
    if metadata.is_populated && !force_type_population {
        return;
    }

    for attribute_metadata in metadata.get_attributes() {
        match reference_source {
            ReferenceSource::Symbol(_, a) => {
                symbol_references.add_symbol_reference_to_symbol(*a, attribute_metadata.get_name(), true)
            }
            ReferenceSource::ClassLikeMember(_, a, b) => {
                symbol_references.add_class_member_reference_to_symbol((*a, *b), attribute_metadata.get_name(), true)
            }
        }
    }

    if let Some(return_type) = metadata.return_type_declaration_metadata.as_mut() {
        populate_union_type(
            &mut return_type.type_union,
            codebase_symbols,
            interner,
            Some(reference_source),
            symbol_references,
            force_type_population,
        );
    }

    if let Some(return_type) = metadata.return_type_metadata.as_mut() {
        populate_union_type(
            &mut return_type.type_union,
            codebase_symbols,
            interner,
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
                interner,
                Some(reference_source),
                symbol_references,
                force_type_population,
            );
        }

        if let Some(type_metadata) = parameter_metadata.out_type.as_mut() {
            populate_union_type(
                &mut type_metadata.type_union,
                codebase_symbols,
                interner,
                Some(reference_source),
                symbol_references,
                force_type_population,
            );
        }

        if let Some(type_metadata) = parameter_metadata.default_type.as_mut() {
            populate_union_type(
                &mut type_metadata.type_union,
                codebase_symbols,
                interner,
                Some(reference_source),
                symbol_references,
                force_type_population,
            );
        }

        for attribute_metadata in &parameter_metadata.attributes {
            match reference_source {
                ReferenceSource::Symbol(in_signature, a) => {
                    symbol_references.add_symbol_reference_to_symbol(*a, attribute_metadata.get_name(), *in_signature)
                }
                ReferenceSource::ClassLikeMember(in_signature, a, b) => symbol_references
                    .add_class_member_reference_to_symbol((*a, *b), attribute_metadata.get_name(), *in_signature),
            }
        }
    }

    for (_, type_parameter_map) in &mut metadata.template_types {
        for (_, type_parameter) in type_parameter_map {
            if force_type_population || type_parameter.needs_population() {
                populate_union_type(
                    type_parameter,
                    codebase_symbols,
                    interner,
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
                        interner,
                        Some(reference_source),
                        symbol_references,
                        force_type_population,
                    );
                }
            }
        }
    }

    if let Some(type_metadata) = metadata.if_this_is_type.as_mut() {
        populate_union_type(
            &mut type_metadata.type_union,
            codebase_symbols,
            interner,
            Some(reference_source),
            symbol_references,
            force_type_population,
        );
    }

    if let Some(type_metadata) = metadata.this_out_type.as_mut() {
        populate_union_type(
            &mut type_metadata.type_union,
            codebase_symbols,
            interner,
            Some(reference_source),
            symbol_references,
            force_type_population,
        );
    }

    for thrown_type in &mut metadata.thrown_types {
        populate_union_type(
            &mut thrown_type.type_union,
            codebase_symbols,
            interner,
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
                    interner,
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
                    interner,
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
                    interner,
                    Some(reference_source),
                    symbol_references,
                    force_type_population,
                );
            }
        }
    }

    metadata.is_populated = true;
}

/// Populates the metadata for a single class-like (class, interface, trait).
///
/// This function is potentially recursive, as it populates parent classes,
/// interfaces, and used traits before processing the current class-like.
/// It uses a remove/insert pattern to handle mutable borrowing across recursive calls.
fn populate_class_like_metadata(
    classlike_name: &StringIdentifier,
    codebase: &mut CodebaseMetadata,
    interner: &ThreadedInterner,
    symbol_references: &mut SymbolReferences,
    safe_symbols: &HashSet<StringIdentifier>,
) {
    if let Some(metadata) = codebase.class_likes.get(classlike_name)
        && metadata.is_populated
    {
        return; // Already done, exit early
    }

    let mut metadata = if let Some(metadata) = codebase.class_likes.remove(classlike_name) {
        metadata
    } else {
        return;
    };

    for attribute_metadata in metadata.get_attributes() {
        symbol_references.add_symbol_reference_to_symbol(metadata.name, attribute_metadata.get_name(), true);
    }

    for property_name in metadata.get_property_names() {
        metadata.add_declaring_property_id(property_name, *classlike_name);
    }

    for method_name in metadata.get_methods().to_vec() {
        metadata.add_declaring_method_id(method_name, *classlike_name);
    }

    let force_repopulation = !safe_symbols.contains(classlike_name);
    for parameter_types in metadata.template_extended_offsets.values_mut() {
        for parameter_type in parameter_types {
            populate_union_type(
                parameter_type,
                &codebase.symbols,
                interner,
                Some(&ReferenceSource::Symbol(true, *classlike_name)),
                symbol_references,
                force_repopulation,
            );
        }
    }

    for trait_name in metadata.get_used_traits() {
        populate_metadata_from_trait(&mut metadata, codebase, interner, trait_name, symbol_references, safe_symbols);
    }

    if let Some(parent_classname) = metadata.get_direct_parent_class() {
        populate_metadata_from_parent_class_like(
            &mut metadata,
            codebase,
            interner,
            parent_classname,
            symbol_references,
            safe_symbols,
        );
    }

    for direct_parent_interface in metadata.get_direct_parent_interfaces() {
        populate_interface_metadata_from_parent_interface(
            &mut metadata,
            codebase,
            interner,
            direct_parent_interface,
            symbol_references,
            safe_symbols,
        );
    }

    // Apply immutability to properties if the class is immutable
    if metadata.is_immutable {
        for property_metadata in metadata.get_properties_mut().values_mut() {
            if !property_metadata.is_static() {
                property_metadata.set_is_readonly(true);
            }
        }
    }

    metadata.mark_as_populated();
    codebase.class_likes.insert(*classlike_name, metadata);
}

/// Populates interface data inherited from a parent interface.
fn populate_interface_metadata_from_parent_interface(
    metadata: &mut ClassLikeMetadata,
    codebase: &mut CodebaseMetadata,
    interner: &ThreadedInterner,
    parent_interface: StringIdentifier,
    symbol_references: &mut SymbolReferences,
    safe_symbols: &HashSet<StringIdentifier>,
) {
    populate_class_like_metadata(&parent_interface, codebase, interner, symbol_references, safe_symbols);

    symbol_references.add_symbol_reference_to_symbol(metadata.name, parent_interface, true);

    let parent_interface_metadata = if let Some(parent_meta) = codebase.class_likes.get(&parent_interface) {
        parent_meta
    } else {
        metadata.invalid_dependencies.push(parent_interface);
        return;
    };

    for (interface_constant_name, interface_constant_metadata) in &parent_interface_metadata.constants {
        if !metadata.constants.contains_key(interface_constant_name) {
            metadata.constants.insert(*interface_constant_name, interface_constant_metadata.clone());
        }
    }

    metadata.add_all_parent_interfaces(parent_interface_metadata.get_all_parent_interfaces().iter().copied());
    metadata.invalid_dependencies.extend(parent_interface_metadata.get_invalid_dependencies().iter().copied());

    if let Some(inheritors) = &parent_interface_metadata.permitted_inheritors {
        metadata.permitted_inheritors.get_or_insert_default().extend(inheritors.iter().copied());
    }

    // Extend template parameters based on the parent interface's templates
    extend_template_parameters(metadata, parent_interface_metadata);
    // Inherit methods (appearing/declaring ids) from the parent interface
    // Pass codebase immutably if possible, or mutably if method inheritance logic needs it
    inherit_methods_from_parent(metadata, parent_interface_metadata, codebase, interner);
    inherit_properties_from_parent(metadata, parent_interface_metadata);
}

/// Populates class-like data inherited from a parent class or trait.
fn populate_metadata_from_parent_class_like(
    metadata: &mut ClassLikeMetadata,
    codebase: &mut CodebaseMetadata,
    interner: &ThreadedInterner,
    parent_class: StringIdentifier,
    symbol_references: &mut SymbolReferences,
    safe_symbols: &HashSet<StringIdentifier>,
) {
    populate_class_like_metadata(&parent_class, codebase, interner, symbol_references, safe_symbols);

    symbol_references.add_symbol_reference_to_symbol(metadata.name, parent_class, true);

    let parent_metadata = if let Some(parent_meta) = codebase.class_likes.get(&parent_class) {
        parent_meta
    } else {
        metadata.invalid_dependencies.push(parent_class);
        return;
    };

    metadata.add_all_parent_classes(parent_metadata.get_all_parent_classes().iter().copied());
    metadata.add_all_parent_interfaces(parent_metadata.get_all_parent_interfaces().iter().copied());
    metadata.add_used_traits(parent_metadata.get_used_traits().iter().copied());
    metadata.invalid_dependencies.extend(parent_metadata.get_invalid_dependencies().iter().copied());

    if let Some(inheritors) = &parent_metadata.permitted_inheritors {
        metadata.permitted_inheritors.get_or_insert_default().extend(inheritors.iter().copied());
    }

    extend_template_parameters(metadata, parent_metadata);

    inherit_methods_from_parent(metadata, parent_metadata, codebase, interner);
    inherit_properties_from_parent(metadata, parent_metadata);

    for (parent_constant_name, parent_constant_metadata) in &parent_metadata.constants {
        if !metadata.constants.contains_key(parent_constant_name) {
            metadata.constants.insert(*parent_constant_name, parent_constant_metadata.clone());
        }
    }

    if parent_metadata.has_consistent_templates {
        metadata.has_consistent_templates = true;
    }
}

/// Populates class-like data inherited from a used trait.
fn populate_metadata_from_trait(
    metadata: &mut ClassLikeMetadata,
    codebase: &mut CodebaseMetadata,
    interner: &ThreadedInterner,
    trait_name: StringIdentifier,
    symbol_references: &mut SymbolReferences,
    safe_symbols: &HashSet<StringIdentifier>,
) {
    populate_class_like_metadata(&trait_name, codebase, interner, symbol_references, safe_symbols);

    symbol_references.add_symbol_reference_to_symbol(metadata.name, trait_name, true);

    let Some(trait_metadata) = codebase.class_likes.get(&trait_name) else {
        metadata.invalid_dependencies.push(trait_name);
        return;
    };

    // Inherit constants (if not already defined)
    for (trait_constant_name, trait_constant_metadata) in &trait_metadata.constants {
        if !metadata.constants.contains_key(trait_constant_name) {
            metadata.constants.insert(*trait_constant_name, trait_constant_metadata.clone());
        }
    }

    // Inherit the trait's parent interfaces (direct parents of the trait become parents of the user)
    metadata.add_all_parent_interfaces(trait_metadata.get_direct_parent_interfaces().iter().copied());
    // Also inherit invalid dependencies from the trait
    metadata.invalid_dependencies.extend(trait_metadata.get_invalid_dependencies().iter().copied());

    // Extend template parameters based on the trait's templates
    extend_template_parameters(metadata, trait_metadata);

    // Inherit methods and properties from the trait
    inherit_methods_from_parent(metadata, trait_metadata, codebase, interner);
    inherit_properties_from_parent(metadata, trait_metadata);
}

/// Inherits method declarations and appearances from a parent class-like.
/// Updates declaring_method_ids, appearing_method_ids, etc.
fn inherit_methods_from_parent(
    metadata: &mut ClassLikeMetadata,
    parent_metadata: &ClassLikeMetadata,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
) {
    let class_like_name = metadata.name;
    let is_trait = metadata.kind.is_trait();

    for (method_name, appearing_class_like) in parent_metadata.get_appearing_method_ids() {
        if metadata.has_appearing_method(method_name) {
            continue;
        }

        metadata
            .appearing_method_ids
            .insert(*method_name, if is_trait { class_like_name } else { *appearing_class_like });

        if codebase.function_likes.contains_key(&(class_like_name, *method_name)) {
            metadata.set_potential_declaring_method_class_names(*method_name, HashSet::from_iter([class_like_name]));
        } else {
            if let Some(parent_potential_method_ids) = parent_metadata.get_potential_declaring_method_id(method_name) {
                metadata.set_potential_declaring_method_class_names(*method_name, parent_potential_method_ids.clone());
            }

            metadata.add_potential_declaring_method(*method_name, class_like_name);
            metadata.add_potential_declaring_method(*method_name, parent_metadata.name);
        }
    }

    let constructor_id = interner.intern("__construct");
    for (method_name, declaring_class) in parent_metadata.get_inheritable_method_ids() {
        if !method_name.eq(&constructor_id) || parent_metadata.has_consistent_constructor {
            if !parent_metadata.kind.is_trait() || is_method_abstract(codebase, interner, declaring_class, method_name)
            {
                metadata.add_overridden_method_parent(*method_name, *declaring_class);
            }

            if let Some(map) = metadata.get_overridden_method_id_mut(method_name)
                && let Some(overridden_method_ids) = parent_metadata.get_overridden_method_id(method_name)
            {
                map.extend(overridden_method_ids.iter().copied());
            }
        }

        let mut aliased_method_names = vec![*method_name];
        if parent_metadata.kind.is_trait() {
            aliased_method_names
                .extend(metadata.get_trait_alias_map().iter().filter(|(_, v)| *v == method_name).map(|(k, _)| *k));
        }

        for aliased_method_name in aliased_method_names {
            let implementing_method_id = metadata.declaring_method_ids.get(&aliased_method_name);
            if let Some(implementing_method_id) = implementing_method_id
                && !is_method_abstract(codebase, interner, implementing_method_id, &aliased_method_name)
            {
                continue;
            }

            metadata.declaring_method_ids.insert(aliased_method_name, *declaring_class);
            metadata.inheritable_method_ids.insert(aliased_method_name, *declaring_class);
        }
    }
}

/// Inherits property declarations and appearances from a parent class-like.
/// Updates declaring_property_ids, appearing_property_ids, etc.
fn inherit_properties_from_parent(metadata: &mut ClassLikeMetadata, parent_metadata: &ClassLikeMetadata) {
    let classlike_name = metadata.name;
    let is_trait = metadata.kind.is_trait();
    let parent_is_trait = parent_metadata.kind.is_trait();

    for (property_name, appearing_classlike) in parent_metadata.get_appearing_property_ids() {
        if metadata.has_appearing_property(property_name) {
            continue;
        }

        if !parent_is_trait
            && let Some(parent_property_metadata) = parent_metadata.get_property(property_name)
            && parent_property_metadata.is_final()
        {
            continue;
        }

        metadata
            .add_appearing_property_id(*property_name, if is_trait { classlike_name } else { *appearing_classlike });
    }

    for (property_name, declaring_classlike) in parent_metadata.get_declaring_property_ids() {
        if metadata.has_declaring_property(property_name) {
            continue;
        }

        if !parent_is_trait
            && let Some(parent_property_metadata) = parent_metadata.get_property(property_name)
            && parent_property_metadata.is_final()
        {
            continue;
        }

        metadata.declaring_property_ids.insert(*property_name, *declaring_classlike);
    }

    for (property_name, inheritable_classlike) in parent_metadata.get_inheritable_property_ids() {
        let mut is_overridable = true;
        if !parent_is_trait {
            if let Some(parent_property_metadata) = parent_metadata.get_property(property_name)
                && parent_property_metadata.is_final()
            {
                is_overridable = false;
            }

            if is_overridable {
                metadata.overridden_property_ids.entry(*property_name).or_default().push(*inheritable_classlike);
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

    if parent_metadata.has_template_types() {
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
            for (parameter_name, parameter_type_map) in parent_metadata.get_template_types() {
                for (_, parameter_type_arc) in parameter_type_map {
                    metadata.add_template_extended_parameter(parent_name, *parameter_name, parameter_type_arc.clone());
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
    template_extended_parameters: &HashMap<StringIdentifier, IndexMap<StringIdentifier, TUnion, RandomState>>,
) -> TUnion {
    if !extended_type.has_template() {
        return extended_type.clone();
    }

    let mut extended_types = Vec::new();

    let mut worklist = extended_type.types.clone();
    while let Some(atomic_type) = worklist.pop() {
        if let TAtomic::GenericParameter(TGenericParameter {
            parameter_name,
            defining_entity: GenericParent::ClassLike(defining_entity),
            ..
        }) = &atomic_type
            && let Some(extended_parameters) = template_extended_parameters.get(defining_entity)
            && let Some(referenced_type) = extended_parameters.get(parameter_name)
        {
            extended_types.extend(referenced_type.types.clone());
            continue;
        }

        extended_types.push(atomic_type);
    }

    TUnion::new(extended_types)
}
