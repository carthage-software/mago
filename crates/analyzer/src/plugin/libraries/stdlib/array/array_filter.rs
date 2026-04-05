//! `array_filter()` return type provider.
//!
//! Narrows the value type(s) of the input array based on the callback — or, with
//! no callback, PHP's default truthy filter. When the input array has a known
//! shape (`array{key: T, ...}`), the shape is preserved in the result but each
//! entry is marked optional (since any entry may be dropped by the filter).

use std::collections::BTreeMap;
use std::sync::Arc;

use mago_codex::assertion::Assertion;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::add_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::combiner::CombinerOptions;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::atomic_comparator;
use mago_codex::ttype::get_array_parameters;
use mago_codex::ttype::get_keyed_array;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta =
    ProviderMeta::new("php::array::array_filter", "array_filter", "Returns filtered array with narrowed value type");

/// Provider for the `array_filter()` function.
///
/// Narrows the value type based on callback assertions or truthiness filtering.
#[derive(Default)]
pub struct ArrayFilterProvider;

impl Provider for ArrayFilterProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for ArrayFilterProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("array_filter")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let array_argument = invocation.get_argument(0, &["array"])?;
        let array_type = context.get_expression_type(array_argument)?;

        let callback_argument = invocation.get_argument(1, &["callback"]);

        let codebase = context.codebase();

        if let Some(callback_arg) = callback_argument {
            let callback_type = context.get_expression_type(callback_arg)?;
            if callback_type.is_null() {
                return filter_arrays(array_type, codebase, filter_falsy_atomics);
            }

            let callback_metadata = context.get_callable_metadata(callback_arg)?;
            if callback_metadata.if_true_assertions.is_empty() {
                return None;
            }

            let first_param = callback_metadata.parameters.first()?;
            let param_name = first_param.get_name().0;
            let assertions = callback_metadata.if_true_assertions.get(&param_name)?.clone();

            return filter_arrays(array_type, codebase, |value_type| {
                let mut narrowed = value_type;
                for assertion in &assertions {
                    narrowed = apply_assertion_to_narrow_type(narrowed, assertion, codebase);
                }
                if narrowed.types.is_empty() { FilterOutcome::Removed } else { FilterOutcome::KeptAsOptional(narrowed) }
            });
        }

        filter_arrays(array_type, codebase, filter_falsy_atomics)
    }
}

/// Apply `filter` to every array atomic in `array_type`, unioning the results.
fn filter_arrays<F>(array_type: &TUnion, codebase: &CodebaseMetadata, filter: F) -> Option<TUnion>
where
    F: Fn(TUnion) -> FilterOutcome,
{
    let mut result: Option<TUnion> = None;
    for atomic in array_type.types.as_ref() {
        let TAtomic::Array(array) = atomic else {
            return None;
        };

        let filtered = filter_atomic_array(array, &filter, codebase)?;
        result = Some(match result {
            Some(acc) => add_union_type(acc, &filtered, codebase, CombinerOptions::default()),
            None => filtered,
        });
    }

    result
}

/// The outcome of filtering a single value type.
enum FilterOutcome {
    Removed,
    KeptAsRequired(TUnion),
    KeptAsOptional(TUnion),
}

fn filter_atomic_array<F>(array: &TArray, filter: &F, codebase: &CodebaseMetadata) -> Option<TUnion>
where
    F: Fn(TUnion) -> FilterOutcome,
{
    if let TArray::Keyed(keyed) = array
        && keyed.known_items.is_some()
    {
        let items = keyed.known_items.as_ref()?;

        let mut new_known_items: BTreeMap<ArrayKey, (bool, TUnion)> = BTreeMap::new();
        for (key, (original_optional, value_type)) in items {
            match filter(value_type.clone()) {
                FilterOutcome::Removed => continue,
                FilterOutcome::KeptAsRequired(filtered) => {
                    new_known_items.insert(*key, (*original_optional, filtered));
                }
                FilterOutcome::KeptAsOptional(filtered) => {
                    new_known_items.insert(*key, (true, filtered));
                }
            }
        }

        let mut parameters = None;
        if let Some((key_param, value_param)) = &keyed.parameters {
            match filter((**value_param).clone()) {
                FilterOutcome::Removed => {}
                FilterOutcome::KeptAsRequired(filtered_values) | FilterOutcome::KeptAsOptional(filtered_values) => {
                    parameters = Some((Arc::clone(key_param), Arc::new(filtered_values)));
                }
            }
        }

        if new_known_items.is_empty() && parameters.is_none() {
            return Some(wrap_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray::new()))));
        }

        let mut result = TKeyedArray::new();
        if !new_known_items.is_empty() {
            result = result.with_known_items(new_known_items);
        }

        if let Some((k, v)) = parameters {
            result = result.with_parameters(k, v);
        }

        return Some(wrap_atomic(TAtomic::Array(TArray::Keyed(result))));
    }

    let (key_type, value_type) = get_array_parameters(array, codebase);
    let filtered_values = match filter(value_type) {
        FilterOutcome::Removed => return None,
        FilterOutcome::KeptAsRequired(v) | FilterOutcome::KeptAsOptional(v) => v,
    };

    Some(get_keyed_array(key_type, filtered_values))
}

fn filter_falsy_atomics(mut value_type: TUnion) -> FilterOutcome {
    if value_type.is_always_truthy() {
        return FilterOutcome::KeptAsRequired(value_type);
    }

    if value_type.is_always_falsy() {
        return FilterOutcome::Removed;
    }

    value_type.types.to_mut().retain(|atomic| !atomic.is_falsy());

    if value_type.types.is_empty() { FilterOutcome::Removed } else { FilterOutcome::KeptAsOptional(value_type) }
}

fn apply_assertion_to_narrow_type(original_type: TUnion, assertion: &Assertion, codebase: &CodebaseMetadata) -> TUnion {
    match assertion {
        Assertion::IsType(atomic) => {
            let mut result = original_type.clone();
            result.types.to_mut().retain(|t| {
                atomic_comparator::is_contained_by(codebase, t, atomic, false, &mut ComparisonResult::default())
            });

            if result.types.is_empty() { original_type } else { result }
        }
        _ => original_type,
    }
}
