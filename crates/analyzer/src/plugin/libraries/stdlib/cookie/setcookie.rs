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

/// Hook for `setcookie()` and `setrawcookie()`.
///
/// These functions have two signatures:
///   1. setcookie(string $name, string $value = "", int $expires_or_options = 0,
///      string $path = "", string $domain = "", bool $secure = false, bool $httponly = false): bool
///   2. setcookie(string $name, string $value = "", array $options = []): bool
///
/// When the 3rd argument is an array, only 3 arguments are allowed.
/// PHP raises a fatal error if additional arguments are passed with the array form.
#[derive(Default)]
pub struct SetCookieHook;

impl Provider for SetCookieHook {
    fn meta() -> &'static ProviderMeta {
        static META: ProviderMeta = ProviderMeta::new(
            "php::cookie::setcookie",
            "setcookie",
            "Validates setcookie/setrawcookie argument combinations.",
        );

        &META
    }
}

impl FunctionCallHook for SetCookieHook {
    fn after_function_call(&self, call: &FunctionCall<'_>, context: &mut HookContext<'_, '_>) -> HookResult<()> {
        let Expression::Identifier(identifier) = call.function else {
            return Ok(());
        };

        let name = identifier.value();
        if !name.eq_ignore_ascii_case("setcookie") && !name.eq_ignore_ascii_case("setrawcookie") {
            return Ok(());
        }

        let arguments = &call.argument_list.arguments;
        if arguments.len() <= 3 {
            return Ok(());
        }

        // Check if the 3rd argument (index 2) is an array type.
        let Some(third_arg) = arguments.get(2) else {
            return Ok(());
        };

        let third_arg_expr = match third_arg {
            Argument::Positional(arg) => arg.value,
            Argument::Named(arg) => {
                let param_name = arg.name.value;
                if param_name != "expires_or_options" && param_name != "options" {
                    return Ok(());
                }
                arg.value
            }
        };

        let Some(third_arg_type) = context.get_expression_type(third_arg_expr) else {
            return Ok(());
        };

        if !third_arg_type.has_array() {
            return Ok(());
        }

        // The 3rd argument is an array, only 3 arguments are allowed.
        // Report the 4th argument as unexpected.
        let Some(fourth_arg) = arguments.get(3) else {
            return Ok(());
        };
        let fourth_arg_span = match fourth_arg {
            Argument::Positional(arg) => arg.value.span(),
            Argument::Named(arg) => arg.span(),
        };

        context.report(
            IssueCode::TooManyArguments,
            Issue::error(format!("Too many arguments provided for function `{name}`."))
                .with_annotation(
                    Annotation::primary(fourth_arg_span).with_message("Unexpected argument provided here"),
                )
                .with_annotation(
                    Annotation::secondary(call.function.span()).with_message("For this function call"),
                )
                .with_note(format!(
                    "When argument #3 (`$expires_or_options`) is an array, `{name}()` expects exactly 3 arguments, but received {}.",
                    arguments.len()
                ))
                .with_help("Remove the extra arguments and pass options in the array instead."),
        );

        Ok(())
    }
}
