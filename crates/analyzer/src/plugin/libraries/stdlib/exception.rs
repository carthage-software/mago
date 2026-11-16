//! `Exception::getCode()` return type provider.

use mago_codex::ttype::add_union_type;
use mago_codex::ttype::combiner::CombinerOptions;
use mago_codex::ttype::union::TUnion;
use mago_word::word;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::method::MethodReturnTypeProvider;
use crate::plugin::provider::method::MethodTarget;

static META: ProviderMeta = ProviderMeta::new(
    "php::exception::getcode",
    "Exception::getCode",
    "Types the code by the `$code` property of the receiver and its subclasses",
);

static TARGETS: [MethodTarget; 1] = [MethodTarget::exact(b"exception", b"getcode")];

#[derive(Default)]
pub struct ExceptionGetCodeProvider;

impl Provider for ExceptionGetCodeProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodReturnTypeProvider for ExceptionGetCodeProvider {
    fn targets() -> &'static [MethodTarget] {
        &TARGETS
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        _class_name: &[u8],
        _method_name: &[u8],
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let class = invocation.inner().target.get_method_context()?.class_like_metadata;
        let codebase = context.codebase();
        let mut code_type = codebase.get_property_type(class.name.as_bytes(), b"$code")?.clone();

        let code = word("$code");
        for descendant in codebase.all_class_like_descendants.get(&class.name).into_iter().flatten() {
            if let Some(redeclared) = codebase
                .class_likes
                .get(descendant)
                .and_then(|metadata| metadata.properties.get(&code))
                .and_then(|property| property.type_metadata.as_ref())
            {
                code_type = add_union_type(code_type, &redeclared.type_union, codebase, CombinerOptions::default());
            }
        }

        Some(code_type)
    }
}
