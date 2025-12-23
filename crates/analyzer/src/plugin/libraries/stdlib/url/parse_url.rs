//! `parse_url()` return type provider.

use std::collections::BTreeMap;

use mago_atom::Atom;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::bool::TBool;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::get_int_range;
use mago_codex::ttype::get_non_empty_string;
use mago_codex::ttype::get_string;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

const PHP_URL_SCHEME: i64 = 0;
const PHP_URL_HOST: i64 = 1;
const PHP_URL_PORT: i64 = 2;
const PHP_URL_USER: i64 = 3;
const PHP_URL_PASS: i64 = 4;
const PHP_URL_PATH: i64 = 5;
const PHP_URL_QUERY: i64 = 6;
const PHP_URL_FRAGMENT: i64 = 7;

static META: ProviderMeta = ProviderMeta::new(
    "php::url::parse_url",
    "parse_url",
    "Returns typed array or component value based on component argument",
);

/// Provider for the `parse_url()` function.
///
/// When called without a component argument, returns `false|array{...}` with URL parts.
/// When called with a specific component constant, returns the appropriate narrowed type.
#[derive(Default)]
pub struct ParseUrlProvider;

impl Provider for ParseUrlProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for ParseUrlProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("parse_url")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let component_arg = invocation.get_argument(1, &["component"]);

        if let Some(arg) = component_arg {
            if let Some(component_type) = context.get_expression_type(arg) {
                let values = collect_component_values(component_type);

                if let Some(values) = values {
                    if values.is_empty() {
                        // No valid component values - fall through to generic return
                    } else {
                        let mut result_types: Vec<TAtomic> = Vec::new();
                        for value in values {
                            let component_ret = get_component_return_type(value);
                            for atomic in component_ret.types.iter() {
                                if !result_types.contains(atomic) {
                                    result_types.push(atomic.clone());
                                }
                            }
                        }

                        return Some(TUnion::from_vec(result_types));
                    }
                }
            }

            // Component provided but not resolvable - return generic union type
            return Some(get_all_components_return_type());
        }

        // No component argument - return full array type
        Some(get_full_array_return_type())
    }
}

/// Collects all possible component values from a type.
/// Returns `None` if the type represents an unbounded set of integers.
/// Returns `Some(vec![])` if the type is empty or has no valid integers.
fn collect_component_values(component_type: &TUnion) -> Option<Vec<i64>> {
    let mut values = Vec::new();

    for atomic in component_type.types.iter() {
        if let TAtomic::Scalar(TScalar::Integer(int_type)) = atomic {
            match *int_type {
                TInteger::Literal(v) => {
                    if !values.contains(&v) {
                        values.push(v);
                    }
                }
                TInteger::Range(from, to) => {
                    let effective_from = from.max(-1);
                    let effective_to = to.min(7);

                    if effective_from <= effective_to {
                        for v in effective_from..=effective_to {
                            if !values.contains(&v) {
                                values.push(v);
                            }
                        }
                    }
                }
                TInteger::From(from) => {
                    let effective_from = from.max(-1);
                    if effective_from <= 7 {
                        for v in effective_from..=7 {
                            if !values.contains(&v) {
                                values.push(v);
                            }
                        }
                    }
                }
                TInteger::To(to) => {
                    let effective_to = to.min(7);
                    if -1 <= effective_to {
                        for v in -1..=effective_to {
                            if !values.contains(&v) {
                                values.push(v);
                            }
                        }
                    }
                }
                TInteger::Unspecified | TInteger::UnspecifiedLiteral => {
                    return None;
                }
            }
        }
    }

    Some(values)
}

/// Returns the type for a specific URL component.
fn get_component_return_type(component: i64) -> TUnion {
    match component {
        PHP_URL_SCHEME | PHP_URL_HOST | PHP_URL_USER | PHP_URL_PASS | PHP_URL_QUERY | PHP_URL_FRAGMENT => {
            // null|non-empty-string
            TUnion::from_vec(vec![TAtomic::Null, TAtomic::Scalar(TScalar::String(TString::non_empty()))])
        }
        PHP_URL_PORT => {
            // null|int<0, 65535>
            TUnion::from_vec(vec![TAtomic::Null, TAtomic::Scalar(TScalar::Integer(TInteger::Range(0, 65535)))])
        }
        PHP_URL_PATH => {
            // null|string (path can be empty string)
            TUnion::from_vec(vec![TAtomic::Null, TAtomic::Scalar(TScalar::String(TString::general()))])
        }
        -1 => {
            // -1 is equivalent to no component - return full array
            get_full_array_return_type()
        }
        _ => TUnion::from_vec(vec![TAtomic::Scalar(TScalar::Bool(TBool::r#false()))]),
    }
}

/// Returns the union of all possible component return types.
/// Used when component type is non-literal (e.g., `int`).
/// `false|null|int<0, 65535>|string|array{...}`
fn get_all_components_return_type() -> TUnion {
    let mut all_components_return_type = get_full_array_return_type();
    all_components_return_type.types.to_mut().push(TAtomic::Null);
    all_components_return_type.types.to_mut().push(TAtomic::Scalar(TScalar::Integer(TInteger::Range(0, 65535))));
    all_components_return_type.types.to_mut().push(TAtomic::Scalar(TScalar::String(TString::general())));

    all_components_return_type
}

/// Returns the full array type when no component is specified.
fn get_full_array_return_type() -> TUnion {
    let mut known_items: BTreeMap<ArrayKey, (bool, TUnion)> = BTreeMap::new();

    let optional_string_fields = ["scheme", "user", "pass", "host", "query", "fragment"];
    for field in optional_string_fields {
        known_items.insert(ArrayKey::String(Atom::from(field)), (true, get_non_empty_string()));
    }

    known_items.insert(ArrayKey::String(Atom::from("port")), (true, get_int_range(Some(0), Some(65535))));
    known_items.insert(ArrayKey::String(Atom::from("path")), (false, get_string()));

    let keyed_array = TKeyedArray::new().with_known_items(known_items);

    TUnion::from_vec(vec![TAtomic::Scalar(TScalar::Bool(TBool::r#false())), TAtomic::Array(TArray::Keyed(keyed_array))])
}
