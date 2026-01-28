use ahash::HashMap;
use itertools::Itertools;
use mago_atom::Atom;
use mago_atom::AtomSet;
use mago_atom::atom;
use mago_reporting::Annotation;
use mago_reporting::Issue;

use crate::identifier::method::MethodIdentifier;
use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::ttype::TypeMetadata;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::alias::TAlias;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::resolution::TypeResolutionContext;
use crate::ttype::union::TUnion;

use super::result::ClassPopulationResult;

/// Populate a single class-like metadata.
///
/// This function takes ownership of the metadata, populates it, and returns
/// a result containing the populated metadata and collected references.
///
/// # Arguments
///
/// * `name` - The class name
/// * `metadata` - The class metadata to populate
/// * `codebase` - The codebase (read-only access)
///
/// # Returns
///
/// A `ClassPopulationResult` containing the populated metadata and references.
pub fn populate(name: Atom, mut metadata: ClassLikeMetadata, codebase: &CodebaseMetadata) -> ClassPopulationResult {
    let mut symbol_references = Vec::new();
    let mut method_contexts = Vec::new();

    for attribute_metadata in &metadata.attributes {
        symbol_references.push((name, attribute_metadata.name, true));
    }

    for property_name in metadata.get_property_names() {
        metadata.add_declaring_property_id(property_name, name);
    }

    for method_name in &metadata.methods {
        let method_id = MethodIdentifier::new(name, *method_name);
        metadata.appearing_method_ids.insert(*method_name, method_id);
        metadata.declaring_method_ids.insert(*method_name, method_id);
    }

    for trait_name in metadata.used_traits.iter().copied().sorted().collect::<Vec<_>>() {
        merge_metadata_from_trait(&mut metadata, codebase, trait_name, &mut symbol_references);
    }

    if let Some(parent_classname) = metadata.direct_parent_class {
        merge_metadata_from_parent_class_like(&mut metadata, codebase, parent_classname, &mut symbol_references);
    }

    let direct_parent_interfaces = metadata.direct_parent_interfaces.iter().copied().sorted().collect::<Vec<_>>();
    for direct_parent_interface in direct_parent_interfaces {
        merge_interface_metadata_from_parent_interface(
            &mut metadata,
            codebase,
            direct_parent_interface,
            &mut symbol_references,
        );
    }

    for required_class in metadata.require_extends.iter().copied().sorted().collect::<Vec<_>>() {
        merge_metadata_from_required_class_like(&mut metadata, codebase, required_class, &mut symbol_references);
    }

    for required_interface in metadata.require_implements.iter().copied().sorted().collect::<Vec<_>>() {
        merge_interface_metadata_from_parent_interface(
            &mut metadata,
            codebase,
            required_interface,
            &mut symbol_references,
        );
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
            let method_id = (name, *method_name);

            let mut updated_context = TypeResolutionContext::default();
            for alias_name in metadata.type_aliases.keys() {
                updated_context = updated_context.with_type_alias(*alias_name);
            }

            method_contexts.push((method_id, updated_context));
        }
    }

    metadata.mark_as_populated();

    ClassPopulationResult { name, metadata, symbol_references, method_contexts }
}

/// Detects circular references in a type definition by walking its dependencies.
fn detect_circular_type_reference(
    type_name: Atom,
    type_metadata: &TypeMetadata,
    all_aliases: &mago_atom::AtomMap<TypeMetadata>,
    visiting: &mut AtomSet,
    path: &mut Vec<String>,
) -> Option<Vec<String>> {
    if visiting.contains(&type_name) {
        let mut cycle_chain = path.clone();
        cycle_chain.push(type_name.to_string());
        return Some(cycle_chain);
    }

    visiting.insert(type_name);
    path.push(type_name.to_string());

    if let Some(cycle) = check_union_for_circular_refs(&type_metadata.type_union, all_aliases, visiting, path) {
        return Some(cycle);
    }

    visiting.remove(&type_name);
    path.pop();
    None
}

fn check_union_for_circular_refs(
    type_union: &TUnion,
    all_aliases: &mago_atom::AtomMap<TypeMetadata>,
    visiting: &mut AtomSet,
    path: &mut Vec<String>,
) -> Option<Vec<String>> {
    let nodes = type_union.get_all_child_nodes();
    for node in nodes {
        if let TypeRef::Atomic(TAtomic::Reference(TReference::Symbol { name, .. })) = node
            && let Some(referenced_type) = all_aliases.get(name)
            && let Some(cycle) = detect_circular_type_reference(*name, referenced_type, all_aliases, visiting, path)
        {
            return Some(cycle);
        }
    }

    None
}

/// Merges interface data inherited from a parent interface.
fn merge_interface_metadata_from_parent_interface(
    metadata: &mut ClassLikeMetadata,
    codebase: &CodebaseMetadata,
    parent_interface: Atom,
    symbol_references: &mut Vec<(Atom, Atom, bool)>,
) {
    symbol_references.push((metadata.name, parent_interface, true));

    let Some(parent_interface_metadata) = codebase.class_likes.get(&parent_interface) else {
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

    extend_template_parameters(metadata, parent_interface_metadata);
    inherit_methods_from_parent(metadata, parent_interface_metadata, codebase);
    inherit_properties_from_parent(metadata, parent_interface_metadata);
}

/// Merges class-like data inherited from a parent class or trait.
fn merge_metadata_from_parent_class_like(
    metadata: &mut ClassLikeMetadata,
    codebase: &CodebaseMetadata,
    parent_class: Atom,
    symbol_references: &mut Vec<(Atom, Atom, bool)>,
) {
    symbol_references.push((metadata.name, parent_class, true));

    let Some(parent_metadata) = codebase.class_likes.get(&parent_class) else {
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

/// Merges class-like data inherited from a required class.
fn merge_metadata_from_required_class_like(
    metadata: &mut ClassLikeMetadata,
    codebase: &CodebaseMetadata,
    parent_class: Atom,
    symbol_references: &mut Vec<(Atom, Atom, bool)>,
) {
    symbol_references.push((metadata.name, parent_class, true));

    let Some(parent_metadata) = codebase.class_likes.get(&parent_class) else {
        metadata.invalid_dependencies.insert(parent_class);
        return;
    };

    metadata.require_extends.extend(parent_metadata.all_parent_classes.iter().copied());
    metadata.require_implements.extend(parent_metadata.all_parent_interfaces.iter().copied());
}

/// Merges class-like data inherited from a used trait.
fn merge_metadata_from_trait(
    metadata: &mut ClassLikeMetadata,
    codebase: &CodebaseMetadata,
    trait_name: Atom,
    symbol_references: &mut Vec<(Atom, Atom, bool)>,
) {
    symbol_references.push((metadata.name, trait_name, true));

    let Some(trait_metadata) = codebase.class_likes.get(&trait_name) else {
        metadata.invalid_dependencies.insert(trait_name);
        return;
    };

    for (trait_constant_name, trait_constant_metadata) in &trait_metadata.constants {
        metadata.trait_constant_ids.insert(*trait_constant_name, trait_name);

        if !metadata.constants.contains_key(trait_constant_name) {
            metadata.constants.insert(*trait_constant_name, trait_constant_metadata.clone());
        }
    }

    metadata.all_parent_interfaces.extend(trait_metadata.direct_parent_interfaces.iter().copied());
    metadata.invalid_dependencies.extend(trait_metadata.invalid_dependencies.iter().copied());
    metadata.add_used_traits(trait_metadata.used_traits.iter().copied());

    extend_template_parameters(metadata, trait_metadata);
    inherit_methods_from_parent(metadata, trait_metadata, codebase);
    inherit_properties_from_parent(metadata, trait_metadata);
}

/// Extends the template parameter map based on parent metadata.
fn extend_template_parameters(metadata: &mut ClassLikeMetadata, parent_metadata: &ClassLikeMetadata) {
    use ahash::RandomState;
    use indexmap::IndexMap;
    use mago_atom::AtomMap;

    let parent_name = parent_metadata.name;

    if parent_metadata.template_types.is_empty() {
        metadata.extend_template_extended_parameters(parent_metadata.template_extended_parameters.clone());
    } else {
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
    }

    fn extend_type(
        extended_type: &TUnion,
        template_extended_parameters: &AtomMap<IndexMap<Atom, TUnion, RandomState>>,
    ) -> TUnion {
        use crate::misc::GenericParent;

        if !extended_type.has_template() {
            return extended_type.clone();
        }

        let mut extended_types = Vec::new();

        let mut worklist = extended_type.types.to_vec();
        while let Some(atomic_type) = worklist.pop() {
            if let TAtomic::GenericParameter(TGenericParameter {
                parameter_name,
                defining_entity: GenericParent::ClassLike(defining_entity),
                ..
            }) = &atomic_type
                && let Some(extended_parameters) = template_extended_parameters.get(defining_entity)
                && let Some(referenced_type) = extended_parameters.get(parameter_name)
            {
                extended_types.extend(referenced_type.types.iter().cloned());
                continue;
            }

            extended_types.push(atomic_type);
        }

        TUnion::from_vec(extended_types)
    }
}

/// Inherits method declarations and appearances from a parent class-like.
fn inherit_methods_from_parent(
    metadata: &mut ClassLikeMetadata,
    parent_metadata: &ClassLikeMetadata,
    codebase: &CodebaseMetadata,
) {
    let class_like_name = metadata.name;
    let parent_is_trait = parent_metadata.kind.is_trait();

    let reverse_alias_map: Option<HashMap<Atom, Vec<Atom>>> = if parent_is_trait && !metadata.trait_alias_map.is_empty()
    {
        let mut map: HashMap<Atom, Vec<Atom>> = HashMap::default();
        for (original, alias) in metadata.get_trait_alias_map() {
            map.entry(*original).or_default().push(*alias);
        }
        Some(map)
    } else {
        None
    };

    for (method_name_lc, appearing_method_id) in &parent_metadata.appearing_method_ids {
        let mut aliased_method_names = vec![*method_name_lc];

        if let Some(ref reverse_map) = reverse_alias_map
            && let Some(aliases) = reverse_map.get(method_name_lc)
        {
            aliased_method_names.extend(aliases.iter().copied());
        }

        for aliased_method_name in aliased_method_names {
            if metadata.has_appearing_method(&aliased_method_name) {
                continue;
            }

            let implemented_method_id = MethodIdentifier::new(class_like_name, aliased_method_name);

            let final_appearing_id = if parent_is_trait { implemented_method_id } else { *appearing_method_id };

            metadata.appearing_method_ids.insert(aliased_method_name, final_appearing_id);
        }
    }

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

        if let Some(ref reverse_map) = reverse_alias_map
            && let Some(aliases) = reverse_map.get(method_name_lc)
        {
            aliased_method_names.extend(aliases.iter().copied());
        }

        for aliased_method_name in aliased_method_names {
            if let Some(implementing_method_id) = metadata.declaring_method_ids.get(&aliased_method_name) {
                let implementing_class = implementing_method_id.get_class_name();
                let implementing_method_name = implementing_method_id.get_method_name();

                if !codebase.method_is_abstract(implementing_class, implementing_method_name)
                    || *implementing_class == class_like_name
                {
                    continue;
                }
            }

            metadata.declaring_method_ids.insert(aliased_method_name, *declaring_method_id);
            metadata.inheritable_method_ids.insert(aliased_method_name, *declaring_method_id);
        }
    }
}

/// Inherits property declarations and appearances from a parent class-like.
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
