mod docblock;
mod hierarchy;
mod merge;
mod methods;
mod properties;
mod signatures;
mod sorter;
mod templates;

use ahash::HashSet;
use mago_atom::AtomMap;
use mago_atom::AtomSet;

use crate::metadata::CodebaseMetadata;
use crate::reference::ReferenceSource;
use crate::reference::SymbolReferences;
use crate::symbol::SymbolIdentifier;
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
    let mut class_likes_to_repopulate = AtomSet::default();
    for (name, metadata) in codebase.class_likes.iter() {
        // Repopulate if not populated OR if user-defined and not marked safe.
        if !metadata.flags.is_populated() || (metadata.flags.is_user_defined() && !safe_symbols.contains(name)) {
            class_likes_to_repopulate.insert(*name);
        }
    }

    for class_like_name in &class_likes_to_repopulate {
        if let Some(classlike_info) = codebase.class_likes.get_mut(class_like_name) {
            classlike_info.flags &= !crate::metadata::flags::MetadataFlags::POPULATED;
            classlike_info.declaring_property_ids.clear();
            classlike_info.appearing_property_ids.clear();
            classlike_info.declaring_method_ids.clear();
            classlike_info.appearing_method_ids.clear();
        }
    }

    let sorted_classes = sorter::sort_class_likes(codebase, &class_likes_to_repopulate);
    for class_name in sorted_classes {
        hierarchy::populate_class_like_metadata_iterative(&class_name, codebase, symbol_references);
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

        signatures::populate_function_like_metadata(
            function_like_metadata,
            &codebase.symbols,
            &reference_source,
            symbol_references,
            force_repopulation,
        );
    }

    for (name, metadata) in codebase.class_likes.iter_mut() {
        let userland_force_repopulation = metadata.flags.is_user_defined() && !safe_symbols.contains(name);

        hierarchy::populate_class_like_types(
            name,
            metadata,
            &codebase.symbols,
            symbol_references,
            userland_force_repopulation,
        );
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

    // Perform docblock inheritance for methods with @inheritDoc or no docblock
    docblock::inherit_method_docblocks(codebase);

    codebase.all_class_like_descendants = all_classlike_descendants;
    codebase.direct_classlike_descendants = direct_classlike_descendants;
    codebase.safe_symbols = safe_symbols;
    codebase.safe_symbol_members = safe_symbol_members;
}
