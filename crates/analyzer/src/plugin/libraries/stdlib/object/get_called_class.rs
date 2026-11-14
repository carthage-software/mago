//! `get_called_class()` return type provider.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeString;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeStringKind;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new(
    "php::object::get_called_class",
    "get_called_class",
    "Returns the class-string of the late static binding class",
);

#[derive(Default)]
pub struct GetCalledClassProvider;

impl Provider for GetCalledClassProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for GetCalledClassProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact(b"get_called_class")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        _invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let class_like = context.get_class_like(context.current_class_name()?)?;
        if class_like.kind.is_trait() {
            return None;
        }

        let mut object = TNamedObject::new(class_like.original_name);
        object.is_static = !class_like.flags.is_final();

        Some(TUnion::from_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::of_type(
            TClassLikeStringKind::Class,
            TAtomic::Object(TObject::Named(object)),
        )))))
    }
}
