//! `strtolower()`, `strtoupper()` and their `mb_` forms on literal strings.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::union::TUnion;
use mago_word::word;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new(
    "php::string::change_case",
    "strtolower",
    "Returns the lowercased or uppercased literals for literal strings",
);

#[derive(Default)]
pub struct ChangeCaseProvider;

impl Provider for ChangeCaseProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for ChangeCaseProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::ExactMultiple(&[b"strtolower", b"strtoupper", b"mb_strtolower", b"mb_strtoupper"])
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        if invocation.argument_count() != 1 {
            return None;
        }

        let argument_type = context.get_expression_type(invocation.get_argument(0, &[b"string"])?)?;
        let name = invocation.function_name().to_ascii_lowercase();
        let multibyte = name.starts_with("mb_");
        let lower = name.ends_with("lower");

        let mut literals = Vec::with_capacity(argument_type.types.len());
        for atomic in argument_type.types.iter() {
            let value = atomic.get_literal_string_value()?;
            if multibyte && !value.is_ascii() {
                return None;
            }

            let changed = if lower { value.to_ascii_lowercase() } else { value.to_ascii_uppercase() };
            literals.push(TAtomic::Scalar(TScalar::literal_string(word(&changed))));
        }

        Some(TUnion::from_vec(literals))
    }
}
