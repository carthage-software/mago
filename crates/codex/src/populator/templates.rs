use ahash::RandomState;
use indexmap::IndexMap;
use mago_atom::Atom;
use mago_atom::AtomMap;

use crate::metadata::class_like::ClassLikeMetadata;
use crate::misc::GenericParent;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::union::TUnion;

/// Extends the template parameter map of `metadata` based on `parent_metadata`.
/// Handles resolving template types inherited from parents/traits.
pub fn extend_template_parameters(metadata: &mut ClassLikeMetadata, parent_metadata: &ClassLikeMetadata) {
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
pub fn extend_type(
    extended_type: &TUnion,
    template_extended_parameters: &AtomMap<IndexMap<Atom, TUnion, RandomState>>,
) -> TUnion {
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
