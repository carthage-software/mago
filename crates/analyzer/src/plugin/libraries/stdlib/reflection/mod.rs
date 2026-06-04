//! Generic-reflection type providers.
//!
//! These providers make `Reflection*<T>` carry useful types derived from the
//! reflected class `T`. Because `ReflectionClass::getMethods()` erases *which*
//! method each element represents, the tightest sound approximation for a
//! single `ReflectionMethod<T>` is "*some* method of `T`": `getName()` returns
//! the union of `T`'s method names, and `invoke()` returns the union of `T`'s
//! method return types.

mod class_has_method;
mod method_get_name;
mod method_invoke;

pub use class_has_method::ReflectionClassHasMethodAssertionProvider;
use mago_codex::ttype::combiner::CombinerOptions;
pub use method_get_name::ReflectionMethodGetNameProvider;
pub use method_invoke::ReflectionMethodInvokeProvider;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::combiner;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
use mago_word::Word;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;

/// The receiver object of the current method call, if it is a single named
/// object (e.g. the `ReflectionMethod<Foo>` in `$method->getName()`).
fn receiver_named_object<'ctx>(invocation: &InvocationInfo<'ctx, '_, '_>) -> Option<&'ctx TNamedObject> {
    let method_context = invocation.inner().target.get_method_context()?;

    match &method_context.class_type {
        StaticClassType::Object(TObject::Named(named)) => Some(named),
        _ => None,
    }
}

/// Resolve the reflected class `T` from the receiver of a `Reflection*<T>`
/// call, but only when the receiver itself is an instance of
/// `expected_receiver` (e.g. `ReflectionMethod`).
///
/// The gate is on the *receiver* class rather than the declaring class passed
/// to the provider: inherited methods like `getName()` (declared on
/// `ReflectionFunctionAbstract`) report the declaring class, so checking the
/// receiver is what distinguishes a `ReflectionMethod` call.
///
/// Returns `None` when there is no instantiated, single named class to read -
/// e.g. an un-parameterized receiver or the default `T of object` - so callers
/// fall back to the declared return type rather than narrowing unsoundly.
fn reflected_class_name(
    context: &ProviderContext<'_, '_, '_>,
    invocation: &InvocationInfo<'_, '_, '_>,
    expected_receiver: &[u8],
) -> Option<Word> {
    let receiver = receiver_named_object(invocation)?;
    if !context.is_instance_of(receiver.get_name().as_bytes(), expected_receiver) {
        return None;
    }

    let parameter = receiver.get_type_parameters()?.first()?;

    Some(parameter.get_single_named_object()?.get_name())
}

/// Build a union of literal-string atomics, one per method that appears on
/// `class_name` (including inherited methods). `None` if the class is unknown
/// or has no methods.
fn method_name_union(codebase: &CodebaseMetadata, class_name: Word) -> Option<TUnion> {
    let class_metadata = codebase.get_class_like(class_name.as_bytes())?;

    let mut atomics = Vec::new();
    for method_id in class_metadata.appearing_method_ids.values() {
        if let Some(metadata) =
            codebase.get_method(method_id.get_class_name().as_bytes(), method_id.get_method_name().as_bytes())
        {
            atomics.push(TAtomic::Scalar(TScalar::literal_string(metadata.original_name)));
        }
    }

    if atomics.is_empty() {
        return None;
    }

    Some(TUnion::from_vec(combiner::combine(atomics, codebase, CombinerOptions::default())))
}

/// Build a union of the return types of every method that appears on
/// `class_name` (including inherited methods). Methods without a declared
/// return type contribute `mixed`. `None` if the class is unknown or has no
/// methods.
fn method_return_union(codebase: &CodebaseMetadata, class_name: Word) -> Option<TUnion> {
    let class_metadata = codebase.get_class_like(class_name.as_bytes())?;

    let mut result: Option<TUnion> = None;
    for method_id in class_metadata.appearing_method_ids.values() {
        let Some(metadata) =
            codebase.get_method(method_id.get_class_name().as_bytes(), method_id.get_method_name().as_bytes())
        else {
            continue;
        };

        let return_type =
            metadata.return_type_metadata.as_ref().map_or_else(get_mixed, |metadata| metadata.type_union.clone());

        result = Some(match result {
            None => return_type,
            Some(accumulated) => combine_union_types(&accumulated, &return_type, codebase, CombinerOptions::default()),
        });
    }

    result
}
