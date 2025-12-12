//! Closure::getCurrent() return type provider.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::get_signature_of_function_like_metadata;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;

use crate::code::IssueCode;
use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::method::MethodReturnTypeProvider;
use crate::plugin::provider::method::MethodTarget;

static META: ProviderMeta = ProviderMeta::new(
    "php::closure::getcurrent",
    "Closure::getCurrent",
    "Returns the current closure's signature type",
);

static TARGETS: [MethodTarget; 1] = [MethodTarget::exact("closure", "getcurrent")];

/// Provider for the `Closure::getCurrent()` method.
///
/// Returns the callable signature of the current closure when called from within
/// a closure or arrow function. Reports an error and returns `never` if called
/// outside a closure.
#[derive(Default)]
pub struct ClosureGetCurrentProvider;

impl Provider for ClosureGetCurrentProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodReturnTypeProvider for ClosureGetCurrentProvider {
    fn targets() -> &'static [MethodTarget] {
        &TARGETS
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        _class_name: &str,
        _method_name: &str,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let scope = context.scope();
        let (Some(closure), Some(closure_identifier)) =
            (scope.get_function_like(), scope.get_function_like_identifier())
        else {
            context.report(
                IssueCode::InvalidStaticMethodCall,
                Issue::error("`Closure::getCurrent()` must be called from within a closure.")
                    .with_annotation(
                        Annotation::primary(invocation.span()).with_message("This call is in the global scope"),
                    )
                    .with_note("This method is only available inside a closure or an arrow function to get a reference to itself, which is useful for recursion.")
                    .with_help("Move this call inside a closure or use a different approach if you are not in a closure context."),
            );

            return Some(get_never());
        };

        if !closure_identifier.is_closure() {
            let kind = closure_identifier.kind_str();

            context.report(
                IssueCode::InvalidStaticMethodCall,
                Issue::error(format!(
                    "`Closure::getCurrent()` must be called from within a closure, but it is currently inside a {kind}."
                ))
                .with_annotation(
                    Annotation::primary(invocation.span())
                        .with_message(format!("This call is inside a {kind}, not a closure")),
                )
                .with_note("This method is only available inside a closure or an arrow function to get a reference to itself, which is useful for recursion.")
                .with_help("Ensure this method is only called within the body of a closure or an arrow function."),
            );

            return Some(get_never());
        };

        let codebase = context.codebase();

        Some(if closure.template_types.is_empty() {
            TUnion::from_atomic(TAtomic::Callable(TCallable::Signature(get_signature_of_function_like_metadata(
                &closure_identifier,
                closure,
                codebase,
                &TypeExpansionOptions::default(),
            ))))
        } else {
            TUnion::from_atomic(TAtomic::Callable(TCallable::Alias(closure_identifier)))
        })
    }
}
