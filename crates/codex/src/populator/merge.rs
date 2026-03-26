use mago_atom::Atom;

use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::populator::methods::inherit_methods_from_parent;
use crate::populator::properties::inherit_properties_from_parent;
use crate::populator::templates::extend_template_parameters;
use crate::reference::SymbolReferences;

/// Merges interface data inherited from a parent interface into the current metadata.
/// Assumes the parent is already populated.
pub fn merge_interface_metadata_from_parent_interface(
    metadata: &mut ClassLikeMetadata,
    codebase: &mut CodebaseMetadata,
    parent_interface: Atom,
    symbol_references: &mut SymbolReferences,
) {
    symbol_references.add_symbol_reference_to_symbol(metadata.name, parent_interface, true);

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
/// Assumes the parent is already populated.
pub fn merge_metadata_from_parent_class_like(
    metadata: &mut ClassLikeMetadata,
    codebase: &mut CodebaseMetadata,
    parent_class: Atom,
    symbol_references: &mut SymbolReferences,
) {
    symbol_references.add_symbol_reference_to_symbol(metadata.name, parent_class, true);

    let Some(parent_metadata) = codebase.class_likes.get(&parent_class) else {
        metadata.invalid_dependencies.insert(parent_class);
        return;
    };

    metadata.all_parent_classes.extend(parent_metadata.all_parent_classes.iter().copied());
    metadata.all_parent_interfaces.extend(parent_metadata.all_parent_interfaces.iter().copied());
    metadata.used_traits.extend(parent_metadata.used_traits.iter().copied());
    metadata.invalid_dependencies.extend(parent_metadata.invalid_dependencies.iter().copied());
    metadata.mixins.extend(parent_metadata.mixins.iter().cloned());

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
/// Assumes the parent is already populated.
pub fn merge_metadata_from_required_class_like(
    metadata: &mut ClassLikeMetadata,
    codebase: &mut CodebaseMetadata,
    parent_class: Atom,
    symbol_references: &mut SymbolReferences,
) {
    symbol_references.add_symbol_reference_to_symbol(metadata.name, parent_class, true);

    let Some(parent_metadata) = codebase.class_likes.get(&parent_class) else {
        metadata.invalid_dependencies.insert(parent_class);
        return;
    };

    metadata.require_extends.extend(parent_metadata.all_parent_classes.iter().copied());
    metadata.require_implements.extend(parent_metadata.all_parent_interfaces.iter().copied());
}

/// Merges class-like data inherited from a used trait.
/// Assumes the trait is already populated.
pub fn merge_metadata_from_trait(
    metadata: &mut ClassLikeMetadata,
    codebase: &mut CodebaseMetadata,
    trait_name: Atom,
    symbol_references: &mut SymbolReferences,
) {
    symbol_references.add_symbol_reference_to_symbol(metadata.name, trait_name, true);

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
    metadata.mixins.extend(trait_metadata.mixins.iter().cloned());
    metadata.add_used_traits(trait_metadata.used_traits.iter().copied());

    extend_template_parameters(metadata, trait_metadata);

    inherit_methods_from_parent(metadata, trait_metadata, codebase);
    inherit_properties_from_parent(metadata, trait_metadata);
}
