//! `array_all()` assertion provider.
//!
//! Lifts a callback predicate's `@assert-if-true` onto the `$array` argument:
//! when `array_all($array, fn ($v) => is_int($v))` returns true, every value
//! of `$array` matches the callback's `if_true_assertion`, so the array can
//! be narrowed to `array<K, NarrowedV>`. Hosts like `assert(array_all(...))`
//! and `if (array_all(...))` consume this through the existing if-true
//! assertion pipeline.

use std::sync::Arc;

use mago_atom::atom;
use mago_codex::assertion::Assertion;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::get_array_parameters;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::libraries::stdlib::array::array_filter::apply_assertion_to_narrow_type;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::assertion::FunctionAssertionProvider;
use crate::plugin::provider::assertion::InvocationAssertions;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta =
    ProviderMeta::new("php::array::array_all", "array_all", "Lifts a callback's if-true assertion onto the array");

#[derive(Default)]
pub struct ArrayAllAssertionProvider;

impl Provider for ArrayAllAssertionProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionAssertionProvider for ArrayAllAssertionProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("array_all")
    }

    fn get_assertions(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<InvocationAssertions> {
        let array_argument = invocation.get_argument(0, &["array"])?;
        let array_type = context.get_expression_type(array_argument)?;

        let callback_argument = invocation.get_argument(1, &["callback"])?;
        let callback_metadata = context.get_callable_metadata(callback_argument)?;
        if callback_metadata.if_true_assertions.is_empty() {
            return None;
        }

        let first_param = callback_metadata.parameters.first()?;
        let param_name = first_param.get_name().0;
        let callback_assertions = callback_metadata.if_true_assertions.get(&param_name)?;

        let codebase = context.codebase();
        let mut narrowed_atomics: Vec<TAtomic> = Vec::new();
        for variant in array_type.types.as_ref().iter() {
            let TAtomic::Array(array) = variant else {
                return None;
            };

            let (key_type, value_type) = get_array_parameters(array, codebase);
            let mut narrowed_value = value_type;
            for assertion in callback_assertions {
                narrowed_value = apply_assertion_to_narrow_type(narrowed_value, assertion, codebase);
            }

            // Preserve known-items shape when present so a `$array{0: int}`
            // input doesn't lose its known entry on narrowing.
            let mut keyed = if let TArray::Keyed(keyed) = array { keyed.clone() } else { TKeyedArray::new() };

            keyed = keyed.with_parameters(Arc::new(key_type), Arc::new(narrowed_value));
            narrowed_atomics.push(TAtomic::Array(TArray::Keyed(keyed)));
        }

        // `Conjunction<Assertion>` carries an AND of constraints; only emit
        // one `IsType` here, so back out if the input union has more than one
        // array variant we'd need to OR.
        let [narrowed_atomic] = narrowed_atomics.try_into().ok()?;

        let assertion = Assertion::IsType(narrowed_atomic);

        let mut result = InvocationAssertions::new();
        result.add_if_true(atom("$array"), vec![assertion]);
        Some(result)
    }
}
