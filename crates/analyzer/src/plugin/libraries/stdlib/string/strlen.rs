//! `strlen()` return type provider.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::get_literal_int;
use mago_codex::ttype::get_non_negative_int;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new(
    "php::string::strlen",
    "strlen",
    "Returns a non-negative integer, or a literal length for literal strings",
);

#[derive(Default)]
pub struct StrlenProvider;

impl Provider for StrlenProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for StrlenProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact(b"strlen")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let string_argument = invocation.get_argument(0, &[b"string"])?;
        let string_argument_type = context.get_expression_type(string_argument)?;
        let string_types = string_argument_type.types.as_ref();
        if let [atomic] = string_types {
            return Some(match atomic.get_literal_string_value() {
                Some(string_literal) => get_literal_int(string_literal.len() as i64),
                None => get_non_negative_int(),
            });
        }

        let mut lengths = Vec::with_capacity(string_types.len());
        for atomic in string_types {
            let Some(string_literal) = atomic.get_literal_string_value() else {
                return Some(get_non_negative_int());
            };

            lengths.push(TAtomic::Scalar(TScalar::literal_int(string_literal.len() as i64)));
        }

        lengths.sort_unstable();
        lengths.dedup();

        Some(TUnion::from_vec(lengths))
    }
}
