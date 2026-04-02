//! `range()` return type provider.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::float::TFloat;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::get_non_empty_list;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta =
    ProviderMeta::new("php::array::range", "range", "Returns non-empty-list with element type based on arguments");

/// Provider for the `range()` function.
///
/// Infers the element type of the returned list based on the types and values
/// of the `$start` and `$end` arguments:
/// - Both ints: `non-empty-list<int>` with appropriate range (positive, negative, etc.)
/// - Either float: `non-empty-list<float>`
/// - Both strings: `non-empty-list<non-empty-string>`
/// - Empty string start: treats as 0 for numeric ranges
#[derive(Default)]
pub struct RangeProvider;

impl Provider for RangeProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for RangeProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("range")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let start_expr = invocation.get_argument(0, &["start"])?;
        let end_expr = invocation.get_argument(1, &["end"])?;

        let start_type = context.get_expression_type(start_expr)?;
        let end_type = context.get_expression_type(end_expr)?;

        if !start_type.is_single() || !end_type.is_single() {
            return None;
        }

        let start_atomic = start_type.get_single();
        let end_atomic = end_type.get_single();

        let element_type = infer_range_element_type(start_atomic, end_atomic)?;

        Some(get_non_empty_list(wrap_atomic(element_type)))
    }
}

fn infer_range_element_type(start: &TAtomic, end: &TAtomic) -> Option<TAtomic> {
    let start_kind = classify(start);
    let end_kind = classify(end);

    match (start_kind, end_kind) {
        (ArgKind::Int(start_val), ArgKind::Int(end_val)) => Some(int_element_type(start_val, end_val)),
        (ArgKind::Float(start_val), ArgKind::Float(end_val)) => Some(float_element_type(start_val, end_val)),
        (ArgKind::Int(start_val), ArgKind::Float(end_val)) => {
            Some(float_element_type(start_val.map(|v| v as f64), end_val))
        }
        (ArgKind::Float(start_val), ArgKind::Int(end_val)) => {
            Some(float_element_type(start_val, end_val.map(|v| v as f64)))
        }
        (ArgKind::NonEmptyString, ArgKind::NonEmptyString) => {
            Some(TAtomic::Scalar(TScalar::String(TString::non_empty())))
        }
        (ArgKind::EmptyString, ArgKind::EmptyString) => Some(TAtomic::Scalar(TScalar::Integer(TInteger::Literal(0)))),
        (ArgKind::EmptyString, ArgKind::Int(end_val)) => Some(int_element_type(Some(0), end_val)),
        (ArgKind::EmptyString, ArgKind::Float(end_val)) => Some(float_element_type(Some(0.0), end_val)),
        (ArgKind::Int(start_val), ArgKind::EmptyString) => Some(int_element_type(start_val, Some(0))),
        (ArgKind::Float(start_val), ArgKind::EmptyString) => Some(float_element_type(start_val, Some(0.0))),
        _ => None,
    }
}

/// Classifies a range argument into its kind.
#[derive(Debug, Clone, Copy)]
enum ArgKind {
    Int(Option<i64>),
    Float(Option<f64>),
    NonEmptyString,
    EmptyString,
    Unknown,
}

fn classify(atomic: &TAtomic) -> ArgKind {
    match atomic {
        TAtomic::Scalar(TScalar::Integer(int)) => ArgKind::Int(int.get_literal_value()),
        TAtomic::Scalar(TScalar::Float(float)) => ArgKind::Float(float.get_literal_value()),
        TAtomic::Scalar(TScalar::String(string)) => {
            if let Some(value) = string.get_known_literal_value() {
                if value.is_empty() { ArgKind::EmptyString } else { ArgKind::NonEmptyString }
            } else if string.is_non_empty() {
                ArgKind::NonEmptyString
            } else {
                ArgKind::Unknown
            }
        }
        _ => ArgKind::Unknown,
    }
}

/// Returns the most precise int type for a range(start, end).
fn int_element_type(start: Option<i64>, end: Option<i64>) -> TAtomic {
    let (min, max) = match (start, end) {
        (Some(a), Some(b)) => (Some(a.min(b)), Some(a.max(b))),
        _ => (None, None),
    };

    let int = match (min, max) {
        (Some(min), Some(max)) if min == max => TInteger::Literal(min),
        (Some(min), Some(max)) if min >= 1 => TInteger::Range(min, max),
        (Some(min), Some(max)) if min >= 0 => TInteger::Range(min, max),
        (Some(min), Some(max)) if max <= -1 => TInteger::Range(min, max),
        (Some(min), Some(max)) if max <= 0 => TInteger::Range(min, max),
        (Some(min), Some(max)) => TInteger::Range(min, max),
        (Some(min), None) if min >= 1 => TInteger::positive(),
        (Some(min), None) if min >= 0 => TInteger::non_negative(),
        (Some(min), None) => TInteger::From(min),
        (None, Some(max)) if max <= -1 => TInteger::negative(),
        (None, Some(max)) if max <= 0 => TInteger::non_positive(),
        (None, Some(max)) => TInteger::To(max),
        (None, None) => TInteger::Unspecified,
    };

    TAtomic::Scalar(TScalar::Integer(int))
}

/// Returns the most precise float type for a range with float arguments.
fn float_element_type(_start: Option<f64>, _end: Option<f64>) -> TAtomic {
    TAtomic::Scalar(TScalar::Float(TFloat::Float))
}
