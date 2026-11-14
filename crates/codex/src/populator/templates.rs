use foldhash::fast::RandomState;
use indexmap::IndexMap;
use mago_word::Word;

use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::misc::GenericParent;
use crate::ttype::template::TemplateResult;
use crate::ttype::template::inferred_type_replacer;
use crate::ttype::union::TUnion;

/// Extends the template parameter map of `metadata` based on `parent_metadata`.
/// Handles resolving template types inherited from parents/traits.
pub fn extend_template_parameters(
    metadata: &mut ClassLikeMetadata,
    parent_metadata: &ClassLikeMetadata,
    codebase: &CodebaseMetadata,
) {
    let parent_name = parent_metadata.name;

    if parent_metadata.template_types.is_empty() {
        // Inherit the parent's extended parameters map directly.
        metadata.extend_template_extended_parameters(parent_metadata.template_extended_parameters.clone());
        inherit_template_extended_paths(metadata, parent_metadata);
    } else {
        metadata.template_extended_parameters.entry(parent_name).or_default();

        let mut parent_path: IndexMap<Word, TUnion, RandomState> = IndexMap::default();
        if let Some(parent_offsets) = metadata.template_extended_offsets.get(&parent_name).cloned() {
            let parent_template_type_names = parent_metadata.get_template_type_names();
            let supplied_count = parent_offsets.len();

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
        } else {
            for (parameter_name, template) in &parent_metadata.template_types {
                let value = template.default.clone().unwrap_or_else(|| template.constraint.clone());
                metadata.add_template_extended_parameter(parent_name, *parameter_name, value.clone());
                parent_path.insert(*parameter_name, value);
            }
        }

        metadata.record_template_extended_path(parent_name, parent_path);
        let mut current_child_extended_params = TemplateResult::default();
        for (defining_entity, parameters) in &metadata.template_extended_parameters {
            for (parameter_name, parameter_type) in parameters {
                current_child_extended_params.add_lower_bound(
                    *parameter_name,
                    GenericParent::ClassLike(*defining_entity),
                    parameter_type.clone(),
                );
            }
        }

        for (grandparent_fqcn, extended_parameters) in &parent_metadata.template_extended_parameters {
            let mut grandparent_path: IndexMap<Word, TUnion, RandomState> = IndexMap::default();
            for (template_name, type_to_resolve_arc) in extended_parameters {
                let resolved_type = extend_type(type_to_resolve_arc, &current_child_extended_params, codebase);

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
                        .insert(*template_name, extend_type(type_to_resolve, &current_child_extended_params, codebase));
                }

                metadata.record_template_extended_path(*grandparent_fqcn, resolved_path);
            }
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

/// Resolves the generic parameters of `extended_type`, at any depth, to the
/// types the child supplies for them.
fn extend_type(extended_type: &TUnion, child_parameters: &TemplateResult, codebase: &CodebaseMetadata) -> TUnion {
    if !extended_type.has_template_types() {
        return extended_type.clone();
    }

    inferred_type_replacer::replace(extended_type, child_parameters, codebase)
}
