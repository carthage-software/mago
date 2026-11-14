//! `strval()` return type provider.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::union::TUnion;

use crate::expression::binary::concat::get_operand_strings;
use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta =
    ProviderMeta::new("php::string::strval", "strval", "Returns the string a `(string)` cast of the value gives");

#[derive(Default)]
pub struct StrvalProvider;

impl Provider for StrvalProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for StrvalProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact(b"strval")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let value_type = context.get_expression_type(invocation.get_argument(0, &[b"value"])?)?;
        if !value_type.types.iter().all(|atomic| matches!(atomic, TAtomic::Scalar(_) | TAtomic::Null)) {
            return None;
        }

        let mut types: Vec<TAtomic> = Vec::new();
        for string in get_operand_strings(value_type) {
            let atomic = TAtomic::Scalar(TScalar::String(string));
            if !types.contains(&atomic) {
                types.push(atomic);
            }
        }

        (!types.is_empty()).then(|| TUnion::from_vec(types))
    }
}
