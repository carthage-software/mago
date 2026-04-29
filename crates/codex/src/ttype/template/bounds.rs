//! Helpers for resolving the *most specific* type a template parameter has
//! been bound to during inference.
//!
//! These are plain bound-resolution helpers that don't depend on the
//! definition-replacement walker — they're used by both
//! [`crate::ttype::template::inferred_type_replacer`] and several analyzer
//! call sites that want to look up the resolved type for a parameter without
//! triggering a full substitution pass.

use std::cmp::Ordering;

use foldhash::HashMap;
use foldhash::HashSet;

use mago_atom::Atom;

use crate::metadata::CodebaseMetadata;
use crate::misc::GenericParent;
use crate::ttype::add_union_type;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::combiner::CombinerOptions;
use crate::ttype::get_mixed;
use crate::ttype::template::TemplateBound;
use crate::ttype::union::TUnion;

/// Walks the bound graph to find the underlying concrete type a template
/// parameter eventually resolves to.
///
/// When a template parameter's bound is itself another template parameter,
/// follows the chain through `lower_bounds` until either a concrete type is
/// found or a cycle is hit (tracked via `visited_entities`).
#[must_use]
pub fn get_root_template_type(
    lower_bounds: &HashMap<Atom, HashMap<GenericParent, Vec<TemplateBound>>>,
    parameter_name: Atom,
    defining_entity: &GenericParent,
    mut visited_entities: HashSet<GenericParent>,
    codebase: &CodebaseMetadata,
) -> Option<TUnion> {
    if visited_entities.contains(defining_entity) {
        return None;
    }

    if let Some(mapped) = lower_bounds.get(&parameter_name)
        && let Some(bounds) = mapped.get(defining_entity)
    {
        let mapped_type = get_most_specific_type_from_bounds(bounds, codebase);

        if !mapped_type.is_single() {
            return Some(mapped_type);
        }

        let first_template = &mapped_type.get_single();

        if let TAtomic::GenericParameter(TGenericParameter { parameter_name, defining_entity, .. }) = first_template {
            visited_entities.insert(*defining_entity);

            return Some(
                get_root_template_type(lower_bounds, *parameter_name, defining_entity, visited_entities, codebase)
                    .unwrap_or(mapped_type),
            );
        }

        return Some(mapped_type);
    }

    None
}

/// Combines the relevant bounds for a template parameter into a single union.
///
/// "Relevant" is defined by [`get_relevant_bounds`] — typically the bounds
/// observed at the shallowest appearance depth, plus any equality bounds.
#[must_use]
pub fn get_most_specific_type_from_bounds(lower_bounds: &[TemplateBound], codebase: &CodebaseMetadata) -> TUnion {
    let relevant_bounds = get_relevant_bounds(lower_bounds);

    if relevant_bounds.is_empty() {
        return get_mixed();
    }

    if relevant_bounds.len() == 1 {
        return relevant_bounds[0].bound_type.clone();
    }

    let mut specific_type = relevant_bounds[0].bound_type.clone();

    for bound in relevant_bounds {
        specific_type = add_union_type(specific_type, &bound.bound_type, codebase, CombinerOptions::default());
    }

    specific_type
}

/// Selects the bounds that should drive the template parameter's resolved
/// type. Bounds at deeper appearance depths are usually overshadowed by
/// shallower ones unless the parameter is invariant (in which case all
/// bounds at the same argument offset are kept).
#[must_use]
pub fn get_relevant_bounds(lower_bounds: &[TemplateBound]) -> Vec<&TemplateBound> {
    let mut lower_bounds = lower_bounds.iter().collect::<Vec<_>>();

    if lower_bounds.len() == 1 {
        return lower_bounds;
    }

    lower_bounds.sort_by(|a, b| a.appearance_depth.partial_cmp(&b.appearance_depth).unwrap_or(Ordering::Equal));

    let mut current_depth = None;
    let mut had_invariant = false;
    let mut last_argument_offset = None;

    let mut applicable_bounds = vec![];

    for template_bound in lower_bounds {
        if let Some(inner) = current_depth {
            if inner != template_bound.appearance_depth && !applicable_bounds.is_empty() {
                if !had_invariant || last_argument_offset == template_bound.argument_offset {
                    break;
                }

                current_depth = Some(template_bound.appearance_depth);
            }
        } else {
            current_depth = Some(template_bound.appearance_depth);
        }

        had_invariant = if had_invariant { true } else { template_bound.equality_bound_classlike.is_some() };

        applicable_bounds.push(template_bound);

        last_argument_offset = template_bound.argument_offset;
    }

    applicable_bounds
}
