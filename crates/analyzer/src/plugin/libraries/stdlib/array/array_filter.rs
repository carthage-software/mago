//! `array_filter()` return type provider.

use mago_codex::assertion::Assertion;
use mago_codex::ttype::get_array_parameters;
use mago_codex::ttype::get_keyed_array;
use mago_codex::ttype::union::TUnion;

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

        let array = array_type.get_single_array()?;
        let codebase = context.codebase();
        let (key_type, mut value_type) = get_array_parameters(array, codebase);

        if let Some(callback_arg) = callback_argument {
            let callback_type = context.get_expression_type(callback_arg)?;

            if !callback_type.is_null() {
                // Try to get metadata from closures OR first-class callables like is_string(...)
                if let Some(callback_metadata) = context.get_callable_metadata(callback_arg)
                    && !callback_metadata.if_true_assertions.is_empty()
                    && let Some(first_param) = callback_metadata.parameters.first()
                {
                    let param_name = &first_param.get_name().0;

                    if let Some(assertions) = callback_metadata.if_true_assertions.get(param_name) {
                        for assertion in assertions {
                            value_type = apply_assertion_to_narrow_type(value_type, assertion, codebase);
                        }

                        if value_type.types.is_empty() {
                            return None;
                        }

                        return Some(get_keyed_array(key_type, value_type));
                    }
                }

                return None;
            }
        }

        value_type.types.to_mut().retain(|atomic| !atomic.is_falsy());

        if value_type.types.is_empty() {
            return None;
        }

        Some(get_keyed_array(key_type, value_type))
    }
}

fn apply_assertion_to_narrow_type(
    original_type: TUnion,
    assertion: &Assertion,
    codebase: &mago_codex::metadata::CodebaseMetadata,
) -> TUnion {
    match assertion {
        Assertion::IsType(atomic) => {
            let mut result = original_type.clone();
            result.types.to_mut().retain(|t| {
                mago_codex::ttype::comparator::atomic_comparator::is_contained_by(
                    codebase,
                    t,
                    atomic,
                    false,
                    &mut mago_codex::ttype::comparator::ComparisonResult::default(),
                )
            });
            if result.types.is_empty() { original_type } else { result }
        }
        _ => original_type,
    }
}
