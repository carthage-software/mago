use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::Argument;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionCall;

use crate::code::IssueCode;
use crate::plugin::context::HookContext;
use crate::plugin::hook::FunctionCallHook;
use crate::plugin::hook::HookResult;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;

/// Hook for `session_set_cookie_params()`.
///
/// This function has two signatures:
///   1. session_set_cookie_params(int $lifetime, ?string $path, ?string $domain,
///      ?bool $secure, ?bool $httponly): bool
///   2. session_set_cookie_params(array $lifetime_or_options): bool
///
/// When the 1st argument is an array, only 1 argument is allowed.
#[derive(Default)]
pub struct SessionSetCookieParamsHook;

impl Provider for SessionSetCookieParamsHook {
    fn meta() -> &'static ProviderMeta {
        static META: ProviderMeta = ProviderMeta::new(
            "php::session::session_set_cookie_params",
            "session_set_cookie_params",
            "Validates session_set_cookie_params argument combinations.",
        );

        &META
    }
}

impl FunctionCallHook for SessionSetCookieParamsHook {
    fn after_function_call(&self, call: &FunctionCall<'_>, context: &mut HookContext<'_, '_>) -> HookResult<()> {
        let Expression::Identifier(identifier) = call.function else {
            return Ok(());
        };

        if !identifier.value().eq_ignore_ascii_case("session_set_cookie_params") {
            return Ok(());
        }

        let arguments = &call.argument_list.arguments;
        if arguments.len() <= 1 {
            return Ok(());
        }

        let Some(first_arg) = arguments.get(0) else {
            return Ok(());
        };

        let first_arg_expr = match first_arg {
            Argument::Positional(arg) => arg.value,
            Argument::Named(arg) => arg.value,
        };

        let Some(first_arg_type) = context.get_expression_type(first_arg_expr) else {
            return Ok(());
        };

        if !first_arg_type.has_array() {
            return Ok(());
        }

        // The 1st argument is an array, only 1 argument is allowed.
        let Some(second_arg) = arguments.get(1) else {
            return Ok(());
        };

        let span = match second_arg {
            Argument::Positional(arg) => arg.value.span(),
            Argument::Named(arg) => arg.span(),
        };

        context.report(
            IssueCode::TooManyArguments,
            Issue::error("Too many arguments provided for function `session_set_cookie_params`.")
                .with_annotation(Annotation::primary(span).with_message("Unexpected argument provided here"))
                .with_annotation(
                    Annotation::secondary(call.function.span()).with_message("For this function call"),
                )
                .with_note(format!(
                    "When the first argument is an array, `session_set_cookie_params()` expects exactly 1 argument, but received {}.",
                    arguments.len()
                ))
                .with_help("Remove the extra arguments and pass options in the array instead."),
        );

        Ok(())
    }
}
