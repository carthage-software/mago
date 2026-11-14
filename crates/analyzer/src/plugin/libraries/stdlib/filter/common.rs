use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::bool::TBool;
use mago_codex::ttype::atomic::scalar::float::TFloat;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::get_mixed_keyed_array;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;

const FILTER_VALIDATE_INT: i64 = 257;
const FILTER_VALIDATE_BOOL: i64 = 258;
const FILTER_VALIDATE_FLOAT: i64 = 259;
const FILTER_VALIDATE_REGEXP: i64 = 272;
const FILTER_VALIDATE_URL: i64 = 273;
const FILTER_VALIDATE_EMAIL: i64 = 274;
const FILTER_VALIDATE_IP: i64 = 275;
const FILTER_VALIDATE_MAC: i64 = 276;
const FILTER_VALIDATE_DOMAIN: i64 = 277;

const FILTER_DEFAULT: i64 = 516;
const FILTER_SANITIZE_ENCODED: i64 = 514;
const FILTER_SANITIZE_SPECIAL_CHARS: i64 = 515;
const FILTER_SANITIZE_EMAIL: i64 = 517;
const FILTER_SANITIZE_URL: i64 = 518;
const FILTER_SANITIZE_NUMBER_INT: i64 = 519;
const FILTER_SANITIZE_NUMBER_FLOAT: i64 = 520;
const FILTER_SANITIZE_FULL_SPECIAL_CHARS: i64 = 522;
const FILTER_SANITIZE_ADD_SLASHES: i64 = 523;

const FILTER_NULL_ON_FAILURE: i64 = 0x0800_0000;
const FILTER_REQUIRE_ARRAY: i64 = 0x0100_0000;
const FILTER_FORCE_ARRAY: i64 = 0x0400_0000;

/// Resolves the return type for filter functions (`filter_var` and `filter_input`).
///
/// The `filter_index` and `options_index` parameters specify which argument positions
/// hold the filter constant and options flag respectively.
pub fn resolve_filter_return_type(
    context: &ProviderContext<'_, '_, '_>,
    invocation: &InvocationInfo<'_, '_, '_>,
    filter_index: usize,
    options_index: usize,
) -> Option<TUnion> {
    let filter_arg = invocation.get_argument(filter_index, &[b"filter"])?;
    let filter_type = context.get_expression_type(filter_arg)?;
    let filter_value = filter_type.get_single_literal_int_value()?;

    let flags = get_flags(context, invocation, options_index);
    let null_on_failure = flags & FILTER_NULL_ON_FAILURE != 0;
    if flags & FILTER_FORCE_ARRAY != 0 {
        return Some(get_mixed_keyed_array());
    }

    if flags & FILTER_REQUIRE_ARRAY != 0 {
        let mut result = get_mixed_keyed_array();
        result.types.to_mut().push(failure_type(null_on_failure));

        return Some(result);
    }

    get_filter_return_type(filter_value, null_on_failure)
}

/// Reads the flags from the options parameter, either a literal integer or the `flags`
/// entry of an options array.
fn get_flags(
    context: &ProviderContext<'_, '_, '_>,
    invocation: &InvocationInfo<'_, '_, '_>,
    options_index: usize,
) -> i64 {
    let Some(options_type) =
        invocation.get_argument(options_index, &[b"options"]).and_then(|options| context.get_expression_type(options))
    else {
        return 0;
    };

    if let Some(flags) = options_type.get_single_literal_int_value() {
        return flags;
    }

    let [TAtomic::Array(TArray::Keyed(options))] = options_type.types.as_ref() else {
        return 0;
    };

    options
        .get_known_items()
        .and_then(|items| {
            items.iter().find_map(|(key, (_, value))| match key {
                ArrayKey::String(name) if name.as_bytes() == b"flags" => value.get_single_literal_int_value(),
                _ => None,
            })
        })
        .unwrap_or(0)
}

/// The failure type: `false` normally, `null` when `FILTER_NULL_ON_FAILURE` is set.
fn failure_type(null_on_failure: bool) -> TAtomic {
    if null_on_failure { TAtomic::Null } else { TAtomic::Scalar(TScalar::Bool(TBool::r#false())) }
}

fn get_filter_return_type(filter: i64, null_on_failure: bool) -> Option<TUnion> {
    match filter {
        FILTER_VALIDATE_INT => Some(TUnion::from_vec(vec![
            TAtomic::Scalar(TScalar::Integer(TInteger::Unspecified)),
            failure_type(null_on_failure),
        ])),
        FILTER_VALIDATE_BOOL => Some(TUnion::from_vec(vec![
            TAtomic::Scalar(TScalar::Bool(TBool::new(None))),
            failure_type(null_on_failure),
        ])),
        FILTER_VALIDATE_FLOAT => {
            Some(TUnion::from_vec(vec![TAtomic::Scalar(TScalar::Float(TFloat::Float)), failure_type(null_on_failure)]))
        }
        FILTER_VALIDATE_URL
        | FILTER_VALIDATE_EMAIL
        | FILTER_VALIDATE_IP
        | FILTER_VALIDATE_MAC
        | FILTER_VALIDATE_DOMAIN
        | FILTER_VALIDATE_REGEXP => Some(TUnion::from_vec(vec![
            TAtomic::Scalar(TScalar::String(TString::general())),
            failure_type(null_on_failure),
        ])),
        // Sanitize filters always return string.
        FILTER_DEFAULT
        | FILTER_SANITIZE_ENCODED
        | FILTER_SANITIZE_SPECIAL_CHARS
        | FILTER_SANITIZE_FULL_SPECIAL_CHARS
        | FILTER_SANITIZE_EMAIL
        | FILTER_SANITIZE_URL
        | FILTER_SANITIZE_NUMBER_INT
        | FILTER_SANITIZE_NUMBER_FLOAT
        | FILTER_SANITIZE_ADD_SLASHES => {
            Some(TUnion::from_vec(vec![TAtomic::Scalar(TScalar::String(TString::general()))]))
        }
        _ => None,
    }
}
