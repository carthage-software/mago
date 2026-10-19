use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;
use mago_word::word;
use rayon::prelude::*;

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
use super::merge::merge_metadata_from_required_interface;
use super::merge::merge_metadata_from_trait;
use super::methods::OverrideSeeds;

#[inline]
fn sorted_atoms(iter: impl Iterator<Item = Word>) -> Vec<Word> {
    let mut values: Vec<_> = iter.collect();
    if values.len() > 1 {
        values.sort_unstable();
    }

    values
}

/// Detects circular references in a type definition by walking its dependencies.
///
/// Returns `Some(chain)` if a cycle is detected, where chain is the path of type names forming the cycle.
/// Returns `None` if no cycle is found.
fn detect_circular_type_reference(
    type_name: Word,
    type_metadata: &TypeMetadata,
    all_aliases: &mago_word::WordMap<TypeMetadata>,
    visiting: &mut WordSet,
    path: &mut Vec<Word>,
) -> Option<Vec<Word>> {
    if visiting.contains(&type_name) {
        let mut cycle_chain = path.clone();
        cycle_chain.push(type_name);
        return Some(cycle_chain);
    }

    visiting.insert(type_name);
    path.push(type_name);

    if let Some(cycle) = check_union_for_circular_refs(&type_metadata.type_union, all_aliases, visiting, path) {
        return Some(cycle);
    }

    visiting.remove(&type_name);
    path.pop();
    None
}

/// Recursively checks a `TUnion` for circular type alias references.
fn check_union_for_circular_refs(
    type_union: &TUnion,
    all_aliases: &mago_word::WordMap<TypeMetadata>,
    visiting: &mut WordSet,
    path: &mut Vec<Word>,
) -> Option<Vec<Word>> {
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
    classlike_name: Word,
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    seeds: &mut OverrideSeeds,
) {
    if let Some(metadata) = codebase.class_likes.get(&classlike_name)
        && metadata.flags.is_populated()
    {
        return; // Already done, exit early
    }

    let Some(mut metadata) = codebase.class_likes.remove(&classlike_name) else {
        return;
    };

    populate_class_like_metadata(classlike_name, &mut metadata, codebase, symbol_references, seeds);
    populate_method_type_aliases(classlike_name, &metadata, codebase);
    codebase.class_likes.insert(classlike_name, metadata);
}

fn populate_class_like_metadata(
    classlike_name: Word,
    metadata: &mut ClassLikeMetadata,
    codebase: &CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    seeds: &mut OverrideSeeds,
) {
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

    for trait_name in sorted_atoms(metadata.used_traits.iter().copied()) {
        merge_metadata_from_trait(metadata, codebase, trait_name, symbol_references, seeds);
    }

    if let Some(parent_classname) = metadata.direct_parent_class {
        merge_metadata_from_parent_class_like(metadata, codebase, parent_classname, symbol_references, seeds);
    }

    let direct_parent_interfaces = sorted_atoms(metadata.direct_parent_interfaces.iter().copied());
    for direct_parent_interface in direct_parent_interfaces {
        merge_interface_metadata_from_parent_interface(
            metadata,
            codebase,
            direct_parent_interface,
            symbol_references,
            seeds,
        );
    }

    for required_class in sorted_atoms(metadata.require_extends.iter().copied()) {
        merge_metadata_from_required_class_like(metadata, codebase, required_class, symbol_references);
    }

    for required_interface in sorted_atoms(metadata.require_implements.iter().copied()) {
        merge_metadata_from_required_interface(metadata, codebase, required_interface, symbol_references, seeds);
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
        if let Some(source_class) = codebase.get_class_like_by_word(source_class_name) {
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
        } else if !codebase.symbols.contains(source_class_name) {
            metadata.issues.push(
                Issue::error(format!("Class `{source_class_name}` not found for type import"))
                    .with_code("unknown-class-in-import-type")
                    .with_annotation(Annotation::primary(import_span))
                    .with_help(format!("Ensure that class `{source_class_name}` is defined and scanned")),
            );
        }
    }

    for (type_name, type_metadata) in &metadata.type_aliases {
        let mut visiting = WordSet::default();
        let mut path = Vec::new();

        if let Some(cycle) =
            detect_circular_type_reference(*type_name, type_metadata, &metadata.type_aliases, &mut visiting, &mut path)
        {
            let cycle_strs: Vec<String> = cycle.iter().map(Word::to_string).collect();
            metadata.issues.push(
                Issue::error(format!("Circular type reference detected: {}", cycle_strs.join(" → ")))
                    .with_code(crate::issue::ScanningIssueKind::CircularTypeImport)
                    .with_annotation(
                        Annotation::primary(type_metadata.span)
                            .with_message("This type is part of a circular reference chain"),
                    )
                    .with_note(format!("The type reference chain creates a cycle: {}", cycle_strs.join(" references ")))
                    .with_help("Reorganize your type definitions to avoid circular dependencies"),
            );
        }
    }

    if !metadata.kind.is_trait()
        && metadata.name != word(b"stringable")
        && metadata.appearing_method_ids.contains_key(&word(b"__tostring"))
        && !metadata.all_parent_interfaces.contains(&word(b"stringable"))
    {
        metadata.add_direct_parent_interface(word(b"stringable"));
    }

    metadata.mark_as_populated();
}

fn populate_method_type_aliases(classlike_name: Word, metadata: &ClassLikeMetadata, codebase: &mut CodebaseMetadata) {
    if !metadata.type_aliases.is_empty() {
        for method_name in &metadata.methods {
            let method_id = (classlike_name, *method_name);
            if let Some(method_metadata) = codebase.function_likes.get_mut(&method_id) {
                let mut updated_context = method_metadata.type_resolution_context.take().unwrap_or_default();
                for alias_name in metadata.type_aliases.keys() {
                    updated_context = updated_context.with_type_alias(*alias_name);
                }

                method_metadata.type_resolution_context = Some(updated_context);
            }
        }
    }
}

/// Populates independent classes together, keeping cyclic edges in their serial order.
pub fn populate_class_likes_parallel(
    sorted_classes: &[Word],
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    retain_references: bool,
) {
    let mut seeds = OverrideSeeds::default();
    if sorted_classes.len() < 128 || rayon::current_num_threads() == 1 {
        for &name in sorted_classes {
            populate_class_like_metadata_iterative(name, codebase, symbol_references, &mut seeds);
        }
        return;
    }

    let positions: WordMap<usize> = sorted_classes.iter().enumerate().map(|(index, name)| (*name, index)).collect();
    let mut dependencies: Option<Vec<ClassDependencies>> =
        Some(sorted_classes.par_iter().map(|name| class_dependencies(*name, codebase, &positions)).collect());
    let mut levels = vec![0; sorted_classes.len()];
    let mut layers: Vec<Vec<usize>> = Vec::new();
    let mut segment_start = 0;
    let mut parallel_classes = 0;
    let mut serial_dependencies = WordSet::default();

    for (index, &name) in sorted_classes.iter().enumerate() {
        let class_dependencies = match &mut dependencies {
            Some(dependencies) => std::mem::take(&mut dependencies[index]),
            None => class_dependencies(name, codebase, &positions),
        };

        let mut level = 0;
        let independent = class_dependencies.is_some_and(|class_dependencies| {
            class_dependencies.into_iter().all(|(dependency, dependency_index)| {
                if serial_dependencies.contains(&dependency) {
                    return false;
                }
                let Some(dependency_index) = dependency_index else {
                    return true;
                };
                if dependency_index >= index {
                    return false;
                }
                if dependency_index >= segment_start {
                    level = level.max(levels[dependency_index] + 1);
                }
                true
            })
        });

        if !independent {
            serial_dependencies.insert(name);
            parallel_classes += populate_hierarchy_segment(
                sorted_classes,
                segment_start,
                index,
                &layers,
                codebase,
                symbol_references,
                retain_references,
            );
            layers.clear();
            populate_class_like_metadata_iterative(name, codebase, symbol_references, &mut seeds);
            segment_start = index + 1;
            dependencies = None;
            continue;
        }

        levels[index] = level;
        if layers.len() <= level {
            layers.resize_with(level + 1, Vec::new);
        }
        layers[level].push(index);
    }

    parallel_classes += populate_hierarchy_segment(
        sorted_classes,
        segment_start,
        sorted_classes.len(),
        &layers,
        codebase,
        symbol_references,
        retain_references,
    );

    tracing::trace!(
        parallel_classes,
        serial_classes = sorted_classes.len() - parallel_classes,
        "Class hierarchy populated."
    );
}

type ClassDependencies = Option<Vec<(Word, Option<usize>)>>;

fn class_dependencies(name: Word, codebase: &CodebaseMetadata, positions: &WordMap<usize>) -> ClassDependencies {
    let metadata = codebase.class_likes.get(&name).filter(|metadata| metadata.name == name)?;
    let trait_requirements = metadata
        .used_traits
        .iter()
        .filter_map(|name| codebase.get_class_like_by_word(*name))
        .flat_map(|trait_metadata| trait_metadata.require_implements.iter().copied());
    let inherited_requirements = metadata
        .require_extends
        .iter()
        .filter_map(|name| codebase.get_class_like_by_word(*name))
        .flat_map(|required_metadata| required_metadata.all_parent_interfaces.iter().copied());

    Some(
        hierarchy_dependencies(metadata)
            .chain(trait_requirements)
            .chain(inherited_requirements)
            .filter_map(|dependency| codebase.resolve_class_like_word(dependency))
            .map(|dependency| (dependency, positions.get(&dependency).copied()))
            .collect(),
    )
}

fn hierarchy_dependencies(metadata: &ClassLikeMetadata) -> impl Iterator<Item = Word> + '_ {
    metadata
        .direct_parent_class
        .into_iter()
        .chain(metadata.used_traits.iter().copied())
        .chain(metadata.direct_parent_interfaces.iter().copied())
        .chain(metadata.require_extends.iter().copied())
        .chain(metadata.require_implements.iter().copied())
        .chain(metadata.imported_type_aliases.values().map(|(name, _, _)| *name))
}

fn populate_hierarchy_segment(
    sorted_classes: &[Word],
    start: usize,
    end: usize,
    layers: &[Vec<usize>],
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    retain_references: bool,
) -> usize {
    if end - start < 128 || !layers.iter().any(|layer| layer.len() >= rayon::current_num_threads()) {
        let mut seeds = OverrideSeeds::default();
        for &name in &sorted_classes[start..end] {
            populate_class_like_metadata_iterative(name, codebase, symbol_references, &mut seeds);
        }
        return 0;
    }

    let mut references = Vec::new();
    let mut spare: Vec<Box<ClassLikeMetadata>> = Vec::new();
    for layer in layers {
        // Placeholders are never modified, so those of the previous layer only need a new identity.
        let mut recycled: Vec<Option<Box<ClassLikeMetadata>>> =
            spare.drain(spare.len().saturating_sub(layer.len())..).map(Some).collect();
        recycled.resize_with(layer.len(), || None);
        let placeholders: Vec<Option<Box<ClassLikeMetadata>>> = layer
            .par_iter()
            .zip(recycled)
            .map(|(&index, recycled)| {
                let metadata = codebase.class_likes.get(&sorted_classes[index])?;
                let mut placeholder = match recycled {
                    Some(mut placeholder) => {
                        placeholder.name = metadata.name;
                        placeholder.original_name = metadata.original_name;
                        placeholder.span = metadata.span;
                        placeholder.name_span = metadata.name_span;
                        placeholder.flags = metadata.flags;
                        placeholder
                    }
                    None => Box::new(ClassLikeMetadata::new(
                        metadata.name,
                        metadata.original_name,
                        metadata.span,
                        metadata.name_span,
                        metadata.flags,
                    )),
                };
                // Keep the kind for stale method IDs; other metadata reads use earlier layers.
                placeholder.kind = metadata.kind;
                Some(placeholder)
            })
            .collect();

        let mut pending = Vec::with_capacity(layer.len());
        for (&index, placeholder) in layer.iter().zip(placeholders) {
            if let Some(placeholder) = placeholder
                && let Some(metadata) = codebase.class_likes.get_mut(&sorted_classes[index])
            {
                pending.push((index, std::mem::replace(metadata, placeholder)));
            }
        }

        references.par_extend(
            pending
                .par_iter_mut()
                .fold(
                    || (SymbolReferences::new(), OverrideSeeds::default()),
                    |(mut references, mut seeds), (index, metadata)| {
                        populate_class_like_metadata(
                            sorted_classes[*index],
                            metadata,
                            codebase,
                            &mut references,
                            &mut seeds,
                        );
                        (references, seeds)
                    },
                )
                .map(|(references, _)| references),
        );

        for (index, metadata) in pending {
            if let Some(target) = codebase.class_likes.get_mut(&sorted_classes[index]) {
                spare.push(std::mem::replace(target, metadata));
            }
        }
    }

    spare.into_par_iter().for_each(drop);

    super::keep_references(symbol_references, references, retain_references);

    for &name in &sorted_classes[start..end] {
        if let Some(metadata) = codebase.class_likes.remove(&name) {
            populate_method_type_aliases(name, &metadata, codebase);
            // Replay the serial remove/insert order: table order can affect trait aliases.
            codebase.class_likes.insert(name, metadata);
        }
    }

    end - start
}

/// Populates types for properties, constants, enum cases, and type aliases within a class-like.
pub fn populate_class_like_types(
    name: Word,
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

    for (property_name, property_metadata) in &mut metadata.magic_properties {
        let property_reference_source = ReferenceSource::ClassLikeMember(true, name, *property_name);

        if let Some(signature) = property_metadata.type_metadata.as_mut() {
            populate_union_type(
                &mut signature.type_union,
                codebase_symbols,
                Some(&property_reference_source),
                symbol_references,
                force_repopulation,
            );
        }

        if let Some(signature) = property_metadata.write_type_metadata.as_mut() {
            populate_union_type(
                &mut signature.type_union,
                codebase_symbols,
                Some(&property_reference_source),
                symbol_references,
                force_repopulation,
            );
        }
    }

    for template in metadata.template_types.values_mut() {
        if force_repopulation || template.constraint.needs_population() {
            populate_union_type(
                &mut template.constraint,
                codebase_symbols,
                Some(&class_like_reference_source),
                symbol_references,
                force_repopulation,
            );
        }

        if let Some(default) = template.default.as_mut()
            && (force_repopulation || default.needs_population())
        {
            populate_union_type(
                default,
                codebase_symbols,
                Some(&class_like_reference_source),
                symbol_references,
                force_repopulation,
            );
        }
    }

    for template in &mut metadata.template_extended_offsets.values_mut().flatten() {
        if force_repopulation || template.needs_population() {
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
        if force_repopulation || p.needs_population() {
            populate_union_type(p, codebase_symbols, None, symbol_references, force_repopulation);
        }
    }

    for p in metadata
        .template_extended_parameter_paths
        .values_mut()
        .flat_map(|paths| paths.iter_mut())
        .flat_map(|m| m.values_mut())
    {
        if force_repopulation || p.needs_population() {
            populate_union_type(p, codebase_symbols, None, symbol_references, force_repopulation);
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

    let declared_mixins = metadata.declared_mixin_count();
    for (index, mixin_type) in metadata.mixins.iter_mut().enumerate() {
        if force_repopulation || mixin_type.type_union.needs_population() {
            populate_union_type(
                &mut mixin_type.type_union,
                codebase_symbols,
                (index < declared_mixins).then_some(&class_like_reference_source),
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
