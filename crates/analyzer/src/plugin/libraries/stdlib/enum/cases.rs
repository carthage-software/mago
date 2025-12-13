//! UnitEnum::cases() return type provider.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::r#enum::TEnum;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::method::MethodReturnTypeProvider;
use crate::plugin::provider::method::MethodTarget;

static META: ProviderMeta =
    ProviderMeta::new("php::enum::cases", "UnitEnum::cases", "Returns non-empty-list for enums with at least one case");

// Use wildcard for class since all enums implement UnitEnum
static TARGETS: [MethodTarget; 1] = [MethodTarget::any_class("cases")];

/// Provider for the `UnitEnum::cases()` method.
///
/// Returns `non-empty-list<EnumType>` for enums with at least one case,
/// or `list<EnumType>` for enums with no cases (edge case).
#[derive(Default)]
pub struct EnumCasesProvider;

impl Provider for EnumCasesProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodReturnTypeProvider for EnumCasesProvider {
    fn targets() -> &'static [MethodTarget] {
        &TARGETS
    }

    fn get_return_type(
        &self,
        _context: &ProviderContext<'_, '_, '_>,
        _class_name: &str,
        _method_name: &str,
        invocation_info: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let class_metadata = invocation_info.invocation.target.get_method_context()?.class_like_metadata;

        if !class_metadata.kind.is_enum() {
            return None;
        }

        let enum_type = TUnion::from_atomic(TAtomic::Object(TObject::Enum(TEnum {
            name: class_metadata.original_name,
            case: None,
        })));

        if !class_metadata.enum_cases.is_empty() {
            Some(TUnion::from_atomic(TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(enum_type))))))
        } else {
            Some(TUnion::from_atomic(TAtomic::Array(TArray::List(TList::new(Box::new(enum_type))))))
        }
    }
}
