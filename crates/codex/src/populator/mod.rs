use foldhash::HashSet;

use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::AtomSet;

use crate::metadata::CodebaseMetadata;
use crate::metadata::constant::ConstantMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::reference::ReferenceSource;
use crate::reference::SymbolReferences;
use crate::symbol::SymbolIdentifier;
use crate::symbol::Symbols;
use crate::ttype::union::populate_union_type;

mod docblock;
mod hierarchy;
mod merge;
mod methods;
mod properties;
mod signatures;
mod sorter;
mod templates;

/// Populates the codebase metadata, resolving types and inheritance.
///
/// This function processes class-likes, function-likes, and constants to:
///
/// - Resolve type signatures (populating `TUnion` and `TAtomic` types).
/// - Calculate inheritance hierarchies (parent classes, interfaces, traits).
/// - Determine method and property origins (declaring vs. appearing).
/// - Build descendant maps for efficient lookup.
pub fn populate_codebase(
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    safe_symbols: AtomSet,
    safe_symbol_members: HashSet<SymbolIdentifier>,
) {
    populate_codebase_inner(codebase, symbol_references, safe_symbols, safe_symbol_members, None)
}

/// Populates the codebase with an optional set of dirty (invalidated) symbols for targeted iteration.
///
/// When `dirty_symbols` is provided, function-like, class-type, and constant repopulation
/// uses targeted `get_mut()` lookups instead of scanning the entire HashMap — O(dirty) instead of O(all).
/// This is critical for incremental mode where only a few symbols change per cycle.
pub fn populate_codebase_targeted(
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    safe_symbols: AtomSet,
    safe_symbol_members: HashSet<SymbolIdentifier>,
    dirty_symbols: HashSet<SymbolIdentifier>,
) {
    populate_codebase_inner(codebase, symbol_references, safe_symbols, safe_symbol_members, Some(dirty_symbols))
}

fn populate_codebase_inner(
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    safe_symbols: AtomSet,
    safe_symbol_members: HashSet<SymbolIdentifier>,
    dirty_symbols: Option<HashSet<SymbolIdentifier>>,
) {
    let mut class_likes_to_repopulate = AtomSet::default();
    if let Some(dirty) = &dirty_symbols {
        let mut dirty_class_names = AtomSet::default();
        for (name, _) in dirty {
            dirty_class_names.insert(*name);
        }

        for class_name in &dirty_class_names {
            if let Some(metadata) = codebase.class_likes.get(class_name)
                && (!metadata.flags.is_populated()
                    || (metadata.flags.is_user_defined() && !safe_symbols.contains(class_name)))
            {
                class_likes_to_repopulate.insert(*class_name);
            }
        }

        // Also repopulate user-defined classes that were invalidated by the
        // cascade (not in safe_symbols) but are not directly in dirty_symbols.
        // This handles e.g. a child class whose parent's method was removed:
        // the child is invalidated (not safe) but its file didn't change,
        // so it's not in dirty_symbols.
        for (class_name, metadata) in &codebase.class_likes {
            if metadata.flags.is_user_defined()
                && !safe_symbols.contains(class_name)
                && !class_likes_to_repopulate.contains(class_name)
            {
                class_likes_to_repopulate.insert(*class_name);
            }
        }
    } else {
        for (name, metadata) in &codebase.class_likes {
            if !metadata.flags.is_populated() || (metadata.flags.is_user_defined() && !safe_symbols.contains(name)) {
                class_likes_to_repopulate.insert(*name);
            }
        }
    }

    for class_like_name in &class_likes_to_repopulate {
        if let Some(classlike_info) = codebase.class_likes.get_mut(class_like_name) {
            classlike_info.flags &= !MetadataFlags::POPULATED;
            classlike_info.declaring_property_ids.clear();
            classlike_info.appearing_property_ids.clear();
            classlike_info.declaring_method_ids.clear();
            classlike_info.appearing_method_ids.clear();
            classlike_info.overridden_method_ids.clear();
            classlike_info.overridden_property_ids.clear();
            classlike_info.invalid_dependencies.clear();
        }
    }

    let sorted_classes = sorter::sort_class_likes(codebase, &class_likes_to_repopulate);
    for class_name in sorted_classes {
        hierarchy::populate_class_like_metadata_iterative(class_name, codebase, symbol_references);
    }

    let incremental = !safe_symbols.is_empty() || !safe_symbol_members.is_empty();

    if let Some(dirty) = &dirty_symbols {
        for dirty_key in dirty {
            if let Some(function_like_metadata) = codebase.function_likes.get_mut(dirty_key) {
                let is_closure_or_arrow = function_like_metadata.get_kind().is_closure()
                    || function_like_metadata.get_kind().is_arrow_function();

                if is_closure_or_arrow {
                    continue;
                }

                let force_repopulation = function_like_metadata.flags.is_user_defined();
                if function_like_metadata.flags.is_populated() && !force_repopulation {
                    continue;
                }

                let reference_source = if dirty_key.1.is_empty() || function_like_metadata.get_kind().is_closure() {
                    ReferenceSource::Symbol(true, dirty_key.0)
                } else {
                    ReferenceSource::ClassLikeMember(true, dirty_key.0, dirty_key.1)
                };

                signatures::populate_function_like_metadata(
                    function_like_metadata,
                    &codebase.symbols,
                    &reference_source,
                    symbol_references,
                    force_repopulation,
                );
            }
        }

        // Also repopulate non-dirty function_likes that are not safe but need repopulation.
        // This handles e.g. a child method when the parent class was re-added:
        // the child method isn't dirty (file didn't change) but it's not safe
        // (parent class changed), so it needs type signature repopulation.
        for (name, function_like_metadata) in &mut codebase.function_likes {
            if dirty.contains(name) {
                continue;
            }

            let is_closure_or_arrow =
                function_like_metadata.get_kind().is_closure() || function_like_metadata.get_kind().is_arrow_function();

            let is_safe = if is_closure_or_arrow {
                true
            } else if name.1.is_empty() {
                safe_symbols.contains(&name.0)
            } else {
                safe_symbol_members.contains(name) || safe_symbols.contains(&name.0)
            };

            let force_repopulation = function_like_metadata.flags.is_user_defined() && !is_safe;
            if function_like_metadata.flags.is_populated() && !force_repopulation {
                continue;
            }

            let reference_source = if name.1.is_empty() || function_like_metadata.get_kind().is_closure() {
                ReferenceSource::Symbol(true, name.0)
            } else {
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
    } else {
        for (name, function_like_metadata) in &mut codebase.function_likes {
            let is_closure_or_arrow =
                function_like_metadata.get_kind().is_closure() || function_like_metadata.get_kind().is_arrow_function();

            let is_safe = if is_closure_or_arrow {
                true
            } else if name.1.is_empty() {
                safe_symbols.contains(&name.0)
            } else {
                safe_symbol_members.contains(name) || safe_symbols.contains(&name.0)
            };

            let force_repopulation = function_like_metadata.flags.is_user_defined() && !is_safe;
            if incremental && function_like_metadata.flags.is_populated() && !force_repopulation {
                continue;
            }

            let reference_source = if name.1.is_empty() || function_like_metadata.get_kind().is_closure() {
                ReferenceSource::Symbol(true, name.0)
            } else {
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
    }

    if let Some(_dirty) = &dirty_symbols {
        for class_name in &class_likes_to_repopulate {
            if let Some(metadata) = codebase.class_likes.get_mut(class_name) {
                hierarchy::populate_class_like_types(
                    *class_name,
                    metadata,
                    &codebase.symbols,
                    symbol_references,
                    true, // force: these are in the repopulate set
                );
            }
        }
    } else {
        for (name, metadata) in &mut codebase.class_likes {
            let force_repopulation = metadata.flags.is_user_defined() && !safe_symbols.contains(name);

            if incremental && metadata.flags.is_populated() && !force_repopulation {
                continue;
            }

            hierarchy::populate_class_like_types(
                *name,
                metadata,
                &codebase.symbols,
                symbol_references,
                force_repopulation,
            );
        }
    }

    if let Some(dirty) = &dirty_symbols {
        let mut dirty_const_names: AtomSet = AtomSet::default();
        for (name, member) in dirty {
            if member.is_empty() {
                dirty_const_names.insert(*name);
            }
        }

        for const_name in &dirty_const_names {
            if let Some(constant) = codebase.constants.get_mut(const_name) {
                let force_repopulation = constant.flags.is_user_defined();
                if constant.flags.is_populated() && !force_repopulation {
                    continue;
                }

                populate_constant(const_name, constant, &codebase.symbols, symbol_references, force_repopulation);
            }
        }
    } else {
        for (name, constant) in &mut codebase.constants {
            let force_repopulation = constant.flags.is_user_defined() && !safe_symbols.contains(name);
            if incremental && constant.flags.is_populated() && !force_repopulation {
                continue;
            }

            populate_constant(name, constant, &codebase.symbols, symbol_references, force_repopulation);
        }
    }

    if !incremental || !class_likes_to_repopulate.is_empty() {
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
                all_classlike_descendants
                    .entry(*parent_class)
                    .or_insert_with(AtomSet::default)
                    .insert(*class_like_name);
            }

            for used_trait in &class_like_metadata.used_traits {
                all_classlike_descendants.entry(*used_trait).or_default().insert(*class_like_name);
            }

            if let Some(parent_class) = &class_like_metadata.direct_parent_class {
                direct_classlike_descendants
                    .entry(*parent_class)
                    .or_insert_with(AtomSet::default)
                    .insert(*class_like_name);
            }
        }

        for (parent_name, children) in &direct_classlike_descendants {
            if let Some(parent_metadata) = codebase.class_likes.get_mut(parent_name) {
                parent_metadata.child_class_likes = Some(children.clone());
            }
        }

        codebase.all_class_like_descendants = all_classlike_descendants;
        codebase.direct_classlike_descendants = direct_classlike_descendants;
    }

    if !incremental || !class_likes_to_repopulate.is_empty() {
        let dirty_classes = if dirty_symbols.is_some() { Some(&class_likes_to_repopulate) } else { None };

        docblock::inherit_method_docblocks(codebase, &safe_symbols, dirty_classes);
    }

    codebase.safe_symbols = safe_symbols;
    codebase.safe_symbol_members = safe_symbol_members;
}

/// Populates a single constant's type metadata.
fn populate_constant(
    name: &Atom,
    constant: &mut ConstantMetadata,
    symbols: &Symbols,
    symbol_references: &mut SymbolReferences,
    force_repopulation: bool,
) {
    for attribute_metadata in &constant.attributes {
        symbol_references.add_symbol_reference_to_symbol(*name, attribute_metadata.name, true);
    }

    if let Some(type_metadata) = &mut constant.type_metadata {
        populate_union_type(
            &mut type_metadata.type_union,
            symbols,
            Some(&ReferenceSource::Symbol(true, *name)),
            symbol_references,
            force_repopulation,
        );
    }

    if let Some(inferred_type) = &mut constant.inferred_type {
        populate_union_type(
            inferred_type,
            symbols,
            Some(&ReferenceSource::Symbol(true, *name)),
            symbol_references,
            force_repopulation,
        );
    }

    constant.flags |= MetadataFlags::POPULATED;
}
