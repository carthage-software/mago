use foldhash::fast::RandomState;
use indexmap::IndexMap;
use mago_word::Word;
use mago_word::WordMap;

use crate::metadata::class_like::ClassLikeMetadata;
use crate::misc::GenericParent;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::union::TUnion;

/// Extends the template parameter map of `metadata` based on `parent_metadata`.
/// Handles resolving template types inherited from parents/traits.
pub fn extend_template_parameters(metadata: &mut ClassLikeMetadata, parent_metadata: &ClassLikeMetadata) {
    let parent_name = parent_metadata.name;

    if parent_metadata.template_types.is_empty() {
        // Inherit the parent's extended parameters map directly.
        metadata.extend_template_extended_parameters(parent_metadata.template_extended_parameters.clone());
        inherit_template_extended_paths(metadata, parent_metadata);
    } else {
        metadata.template_extended_parameters.entry(parent_name).or_default();

        if let Some(parent_offsets) = metadata.template_extended_offsets.get(&parent_name).cloned() {
            let parent_template_type_names = parent_metadata.get_template_type_names();
            let supplied_count = parent_offsets.len();

            let mut parent_path: IndexMap<Word, TUnion, RandomState> = IndexMap::default();
            for (i, extended_type_arc) in parent_offsets.into_iter().enumerate() {
                if let Some(mapped_name) = parent_template_type_names.get(i).copied() {
                    metadata.add_template_extended_parameter(parent_name, mapped_name, extended_type_arc.clone());
                    parent_path.insert(mapped_name, extended_type_arc);
                }
            }

            for (parameter_name, template) in parent_metadata.template_types.iter().skip(supplied_count) {
                let Some(default) = &template.default else {
                    break;
                };

                metadata.add_template_extended_parameter(parent_name, *parameter_name, default.clone());
                parent_path.insert(*parameter_name, default.clone());
            }

            metadata.record_template_extended_path(parent_name, parent_path);
            let current_child_extended_params = metadata.template_extended_parameters.clone();
            for (grandparent_fqcn, extended_parameters) in &parent_metadata.template_extended_parameters {
                let mut grandparent_path: IndexMap<Word, TUnion, RandomState> = IndexMap::default();
                for (template_name, type_to_resolve_arc) in extended_parameters {
                    let resolved_type = extend_type(type_to_resolve_arc, &current_child_extended_params);

                    metadata.add_template_extended_parameter(*grandparent_fqcn, *template_name, resolved_type.clone());
                    grandparent_path.insert(*template_name, resolved_type);
                }

                metadata.record_template_extended_path(*grandparent_fqcn, grandparent_path);
            }

            for (grandparent_fqcn, parameterizations) in &parent_metadata.template_extended_parameter_paths {
                for parameterization in parameterizations {
                    let mut resolved_path: IndexMap<Word, TUnion, RandomState> = IndexMap::default();
                    for (template_name, type_to_resolve) in parameterization {
                        resolved_path
                            .insert(*template_name, extend_type(type_to_resolve, &current_child_extended_params));
                    }

                    metadata.record_template_extended_path(*grandparent_fqcn, resolved_path);
                }
            }
        } else {
            let mut parent_path: IndexMap<Word, TUnion, RandomState> = IndexMap::default();
            for (parameter_name, template) in &parent_metadata.template_types {
                let value = template.default.clone().unwrap_or_else(|| template.constraint.clone());
                metadata.add_template_extended_parameter(parent_name, *parameter_name, value.clone());
                parent_path.insert(*parameter_name, value);
            }

            metadata.record_template_extended_path(parent_name, parent_path);
            metadata.extend_template_extended_parameters(parent_metadata.template_extended_parameters.clone());
            inherit_template_extended_paths(metadata, parent_metadata);
        }
    }
}

/// Folds a parent's recorded ancestor parameterizations into the child, so that
/// diamond inheritance through a non-generic parent preserves every path.
fn inherit_template_extended_paths(metadata: &mut ClassLikeMetadata, parent_metadata: &ClassLikeMetadata) {
    for (ancestor, parameterization) in &parent_metadata.template_extended_parameters {
        metadata.record_template_extended_path(*ancestor, parameterization.clone());
    }

    for (ancestor, parameterizations) in &parent_metadata.template_extended_parameter_paths {
        for parameterization in parameterizations {
            metadata.record_template_extended_path(*ancestor, parameterization.clone());
        }
    }
}

/// Resolves a `TUnion` that might contain generic parameters, using the provided
/// extended parameter map.
///
/// Example: If `extended_type` is `T` (generic param) and `template_extended_parameters`
/// maps `T` defined on `ParentClass` to `string`, this returns a `TUnion` containing `string`.
pub fn extend_type(
    extended_type: &TUnion,
    template_extended_parameters: &WordMap<IndexMap<Word, TUnion, RandomState>>,
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
