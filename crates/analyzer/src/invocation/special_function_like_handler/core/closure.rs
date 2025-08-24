use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::get_signature_of_function_like_metadata;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;

use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::invocation::Invocation;
use crate::invocation::special_function_like_handler::SpecialFunctionLikeHandlerTrait;

#[derive(Debug)]
pub struct GetCurrentClosureMethodHandler;

impl SpecialFunctionLikeHandlerTrait for GetCurrentClosureMethodHandler {
    fn get_return_type<'ctx, 'ast, 'arena>(
        &self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &BlockContext<'ctx>,
        _artifacts: &AnalysisArtifacts,
        function_like_name: &str,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> Option<TUnion> {
        if function_like_name != "closure::getcurrent" {
            return None;
        }

        let (Some(closure), Some(closure_identifier)) =
            (block_context.scope.get_function_like(), block_context.scope.get_function_like_identifier())
        else {
            context.collector.report_with_code(
                IssueCode::InvalidStaticMethodCall,
                Issue::error("`Closure::getCurrent()` must be called from within a closure.")
                    .with_annotation(Annotation::primary(invocation.span).with_message("This call is in the global scope"))
                    .with_note("This method is only available inside a closure or an arrow function to get a reference to itself, which is useful for recursion.")
                    .with_help("Move this call inside a closure or use a different approach if you are not in a closure context."),
            );

            return Some(get_never());
        };

        if !closure_identifier.is_closure() {
            let kind = closure_identifier.kind_str();

            context.collector.report_with_code(
                IssueCode::InvalidStaticMethodCall,
                Issue::error(format!(
                    "`Closure::getCurrent()` must be called from within a closure, but it is currently inside a {kind}."
                ))
                .with_annotation(
                    Annotation::primary(invocation.span)
                        .with_message(format!("This call is inside a {kind}, not a closure")),
                )
                .with_note("This method is only available inside a closure or an arrow function to get a reference to itself, which is useful for recursion.")
                .with_help("Ensure this method is only called within the body of a closure or an arrow function."),
            );

            return Some(get_never());
        };

        Some(if closure.template_types.is_empty() {
            TUnion::from_atomic(TAtomic::Callable(TCallable::Signature(get_signature_of_function_like_metadata(
                &closure_identifier,
                closure,
                context.codebase,
                &TypeExpansionOptions::default(),
            ))))
        } else {
            TUnion::from_atomic(TAtomic::Callable(TCallable::Alias(closure_identifier)))
        })
    }
}
