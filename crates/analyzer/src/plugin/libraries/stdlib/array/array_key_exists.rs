//! `array_key_exists()` / `key_exists()` return type provider.
//!
//! When called with a literal key on an array whose shape is fully known and does not
//! contain that key, narrow the return type from `bool` to `false`. This lets the
//! generic "impossible condition" reporter fire on the enclosing `if`, matching the
//! behavior already seen for `empty()` and `isset()` on the same missing key.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::get_false;
use mago_codex::ttype::union::TUnion;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Literal;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new(
    "php::array::array_key_exists",
    "array_key_exists",
    "Narrows the return to `false` when the literal key is provably absent from the array's known shape",
);

#[derive(Default)]
pub struct ArrayKeyExistsProvider;

impl Provider for ArrayKeyExistsProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for ArrayKeyExistsProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::ExactMultiple(&["array_key_exists", "key_exists"])
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let key_expr = invocation.get_argument(0, &["key"])?;
        let array_expr = invocation.get_argument(1, &["array"])?;

        let literal_key = literal_array_key_from_expression(key_expr)?;
        let array_type = context.get_expression_type(array_expr)?;

        if array_type.types.is_empty() {
            return None;
        }

        for atomic in array_type.types.as_ref() {
            match atomic {
                TAtomic::Array(TArray::Keyed(keyed)) => {
                    if keyed.get_generic_parameters().is_some() {
                        return None;
                    }

                    match keyed.get_known_items() {
                        Some(known) if !known.contains_key(&literal_key) => {}
                        _ => return None,
                    }
                }
                TAtomic::Array(TArray::List(list)) => {
                    let ArrayKey::Integer(i) = literal_key else {
                        return None;
                    };

                    if i < 0 {
                        continue;
                    }

                    if !list.element_type.is_never() {
                        return None;
                    }

                    if let Some(known_elements) = &list.known_elements
                        && known_elements.contains_key(&(i as usize))
                    {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        Some(get_false())
    }
}

fn literal_array_key_from_expression(expr: &Expression<'_>) -> Option<ArrayKey> {
    match expr {
        Expression::Literal(Literal::String(s)) => s.value.map(|v| ArrayKey::String(mago_atom::atom(v))),
        Expression::Literal(Literal::Integer(i)) => i.value.and_then(|v| i64::try_from(v).ok()).map(ArrayKey::Integer),
        _ => None,
    }
}
