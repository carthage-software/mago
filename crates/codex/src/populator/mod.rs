//! Codebase population system.
//!
//! # Architecture
//!
//! The population process is divided into phases:
//!
//! 1. **Class Population** - Process classes level-by-level based on dependency depth.
//!    Classes at the same level have no dependencies on each other and can be
//!    processed in parallel.
//! 2. **Function Population** - Populate function-like metadata.
//! 3. **Type Population** - Populate class types.
//! 4. **Constant Population** - Populate constants.
//! 5. **Docblock Inheritance** - Inherit docblocks from parent methods.
//! 6. **Descendant Maps** - Build class descendant maps for efficient lookup.

mod class_like;
mod docblock;
mod hierarchy;
mod result;
mod signatures;

use ahash::HashSet;
use mago_atom::AtomMap;
use mago_atom::AtomSet;
use rayon::prelude::*;

use crate::dependency::DependencyGraph;
use crate::metadata::CodebaseMetadata;
use crate::populator::docblock::inherit_method_docblocks;
use crate::populator::hierarchy::populate_class_like_types;
use crate::populator::signatures::populate_function_like_metadata;
use crate::reference::ReferenceSource;
use crate::reference::SymbolReferences;
use crate::symbol::SymbolIdentifier;
use crate::ttype::union::populate_union_type;

pub use result::AccumulatedReferences;
pub use result::ClassPopulationResult;

/// Populates the codebase metadata using parallel processing.
///
/// This function processes class-likes, function-likes, and constants to:
///
/// - Resolve type signatures (populating `TUnion` and `TAtomic` types).
/// - Calculate inheritance hierarchies (parent classes, interfaces, traits).
/// - Determine method and property origins (declaring vs. appearing).
/// - Build descendant maps for efficient lookup.
///
/// # Arguments
///
/// * `codebase` - The codebase metadata to populate
/// * `symbol_references` - Reference tracker for symbol dependencies
/// * `safe_symbols` - Set of symbols that don't need repopulation
/// * `safe_symbol_members` - Set of symbol members that don't need repopulation
pub fn populate_codebase(
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    safe_symbols: AtomSet,
    safe_symbol_members: HashSet<SymbolIdentifier>,
) {
    let classes_to_repopulate = identify_classes_to_repopulate(codebase, &safe_symbols);

    reset_class_flags(codebase, &classes_to_repopulate);

    let graph = DependencyGraph::build(codebase, &classes_to_repopulate);

    for level in graph.levels() {
        if level.is_empty() {
            continue;
        }

        let level_classes: Vec<_> =
            level.iter().filter_map(|name| codebase.class_likes.remove(name).map(|m| (*name, m))).collect();

        let results: Vec<ClassPopulationResult> = level_classes
            .into_par_iter()
            .map(|(name, metadata)| class_like::populate(name, metadata, codebase))
            .collect();

        for result in results {
            codebase.class_likes.insert(result.name, result.metadata);

            for (from, to, userland) in result.symbol_references {
                symbol_references.add_symbol_reference_to_symbol(from, to, userland);
            }

            for ((class_name, method_name), context) in result.method_contexts {
                if let Some(method_metadata) = codebase.function_likes.get_mut(&(class_name, method_name)) {
                    if let Some(existing) = method_metadata.type_resolution_context.take() {
                        let mut updated = existing;
                        updated.merge(context);
                        method_metadata.type_resolution_context = Some(updated);
                    } else {
                        method_metadata.type_resolution_context = Some(context);
                    }
                }
            }
        }
    }

    populate_function_likes(codebase, symbol_references, &safe_symbols);
    populate_class_types(codebase, symbol_references, &safe_symbols);
    populate_constants(codebase, symbol_references, &safe_symbols);
    inherit_method_docblocks(codebase);
    build_descendant_maps(codebase);

    codebase.safe_symbols = safe_symbols;
    codebase.safe_symbol_members = safe_symbol_members;
}

/// Identify classes that need (re)population.
fn identify_classes_to_repopulate(codebase: &CodebaseMetadata, safe_symbols: &AtomSet) -> AtomSet {
    let mut classes_to_repopulate = AtomSet::default();

    for (name, metadata) in &codebase.class_likes {
        if !metadata.flags.is_populated() || (metadata.flags.is_user_defined() && !safe_symbols.contains(name)) {
            classes_to_repopulate.insert(*name);
        }
    }

    classes_to_repopulate
}

fn reset_class_flags(codebase: &mut CodebaseMetadata, classes_to_repopulate: &AtomSet) {
    for class_like_name in classes_to_repopulate {
        if let Some(classlike_info) = codebase.class_likes.get_mut(class_like_name) {
            classlike_info.flags &= !crate::metadata::flags::MetadataFlags::POPULATED;
            classlike_info.declaring_property_ids.clear();
            classlike_info.appearing_property_ids.clear();
            classlike_info.declaring_method_ids.clear();
            classlike_info.appearing_method_ids.clear();
        }
    }
}

fn populate_function_likes(
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    safe_symbols: &AtomSet,
) {
    let function_names: Vec<_> = codebase.function_likes.keys().cloned().collect();

    for name in function_names {
        let force_repopulation = {
            let Some(function_like_metadata) = codebase.function_likes.get(&name) else {
                continue;
            };

            function_like_metadata.flags.is_user_defined() && !safe_symbols.contains(&name.0)
        };

        let reference_source = {
            let Some(function_like_metadata) = codebase.function_likes.get(&name) else {
                continue;
            };

            if name.1.is_empty() || function_like_metadata.get_kind().is_closure() {
                ReferenceSource::Symbol(true, name.0)
            } else {
                ReferenceSource::ClassLikeMember(true, name.0, name.1)
            }
        };

        if let Some(function_like_metadata) = codebase.function_likes.get_mut(&name) {
            populate_function_like_metadata(
                function_like_metadata,
                &codebase.symbols,
                &reference_source,
                symbol_references,
                force_repopulation,
            );
        }
    }
}

fn populate_class_types(
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    safe_symbols: &AtomSet,
) {
    let class_names: Vec<_> = codebase.class_likes.keys().cloned().collect();

    for name in class_names {
        let force_repopulation = {
            let Some(metadata) = codebase.class_likes.get(&name) else {
                continue;
            };

            metadata.flags.is_user_defined() && !safe_symbols.contains(&name)
        };

        if let Some(metadata) = codebase.class_likes.get_mut(&name) {
            populate_class_like_types(name, metadata, &codebase.symbols, symbol_references, force_repopulation);
        }
    }
}

fn populate_constants(
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    safe_symbols: &AtomSet,
) {
    let constant_names: Vec<_> = codebase.constants.keys().cloned().collect();

    for name in constant_names {
        let Some(constant) = codebase.constants.get_mut(&name) else {
            continue;
        };

        for attribute_metadata in &constant.attributes {
            symbol_references.add_symbol_reference_to_symbol(name, attribute_metadata.name, true);
        }

        if let Some(type_metadata) = &mut constant.type_metadata {
            populate_union_type(
                &mut type_metadata.type_union,
                &codebase.symbols,
                Some(&ReferenceSource::Symbol(true, name)),
                symbol_references,
                !safe_symbols.contains(&name),
            );
        }

        if let Some(inferred_type) = &mut constant.inferred_type {
            populate_union_type(
                inferred_type,
                &codebase.symbols,
                Some(&ReferenceSource::Symbol(true, name)),
                symbol_references,
                !safe_symbols.contains(&name),
            );
        }
    }
}

fn build_descendant_maps(codebase: &mut CodebaseMetadata) {
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
}
