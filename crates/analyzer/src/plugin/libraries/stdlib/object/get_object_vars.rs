use std::collections::BTreeMap;
use std::sync::Arc;

use mago_codex::context::ScopeContext;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::property::PropertyMetadata;
use mago_codex::reference::SymbolReferences;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
use mago_word::Word;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;
use crate::visibility::is_visible_from_scope;

static META: ProviderMeta = ProviderMeta::new(
    "php::object::get_object_vars",
    "get_object_vars",
    "Returns the object's properties visible from the calling scope as a keyed array",
);

#[derive(Default)]
pub struct GetObjectVarsProvider;

impl Provider for GetObjectVarsProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for GetObjectVarsProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact(b"get_object_vars")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let argument = invocation.get_argument(0, &[b"object"])?;
        let argument_type = context.get_expression_type(argument)?;
        let object = argument_type.get_single_named_object()?;
        let class_metadata = context.get_class_like(object.name)?;

        let mut known_items: BTreeMap<ArrayKey, (bool, TUnion)> = BTreeMap::new();
        for_each_visible_instance_property(
            context.codebase(),
            class_metadata,
            context.current_class_name(),
            |property_name, _, property| {
                let Some(name) = property_name.as_bytes().strip_prefix(b"$") else {
                    return;
                };

                let property_type = property
                    .type_metadata
                    .as_ref()
                    .map(|metadata| metadata.type_union.clone())
                    .unwrap_or_else(get_mixed);

                known_items.insert(ArrayKey::String(mago_word::word(name)), (false, property_type));
            },
        );

        if known_items.is_empty() {
            return None;
        }

        let mut keyed_array = TKeyedArray::new();
        keyed_array.known_items = Some(known_items.into());
        keyed_array.non_empty = true;

        if !class_metadata.flags.is_final() || !class_metadata.flags.is_readonly() {
            keyed_array.parameters = Some((Arc::new(get_arraykey()), Arc::new(get_mixed())));
        }

        Some(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed_array))))
    }
}

/// Records a read of every property that `get_object_vars()` returns for `object_type`.
///
/// Without this, properties that are only read through `get_object_vars($this)`
/// (for example in `jsonSerialize()`) are reported as unused or write-only.
pub(crate) fn record_property_reads(
    codebase: &CodebaseMetadata,
    scope: &ScopeContext<'_>,
    symbol_references: &mut SymbolReferences,
    object_type: &TUnion,
) {
    let current_class = scope.get_class_like_name();

    for atomic in object_type.types.as_ref() {
        let intersection_types = atomic.get_intersection_types().into_iter().flatten();

        for object_atomic in std::iter::once(atomic).chain(intersection_types) {
            let TAtomic::Object(TObject::Named(named)) = object_atomic else {
                continue;
            };

            let Some(class_metadata) = codebase.get_class_like_by_name(named.name) else {
                continue;
            };

            for_each_visible_instance_property(
                codebase,
                class_metadata,
                current_class,
                |property_name, declaring_class, _| {
                    symbol_references.add_reference_for_property_read(scope, declaring_class, property_name);
                },
            );
        }
    }
}

/// Calls `visit` with `(property name, declaring class, property)` for every instance property
/// of `class_metadata` that `get_object_vars()` returns when called from `current_class`.
fn for_each_visible_instance_property<'ctx>(
    codebase: &'ctx CodebaseMetadata,
    class_metadata: &'ctx ClassLikeMetadata,
    current_class: Option<Word>,
    mut visit: impl FnMut(Word, Word, &'ctx PropertyMetadata),
) {
    if class_metadata.kind.is_enum() {
        return;
    }

    for (&property_name, &appearing_class) in &class_metadata.appearing_property_ids {
        let Some(property) = codebase.get_property(appearing_class.as_bytes(), property_name.as_bytes()) else {
            continue;
        };

        if property.flags.is_static() || property.flags.is_virtual_property() {
            continue;
        }

        if !is_visible_from_scope(codebase, property.read_visibility, appearing_class.as_bytes(), current_class) {
            continue;
        }

        let declaring_class =
            class_metadata.declaring_property_ids.get(&property_name).copied().unwrap_or(appearing_class);

        visit(property_name, declaring_class, property);
    }
}
