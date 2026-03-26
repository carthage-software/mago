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

/// Hook for `session_set_save_handler()`.
///
/// This function has two signatures:
///   1. session_set_save_handler(callable, callable, callable, callable, callable, callable,
///      ?callable, ?callable, ?callable): bool
///   2. session_set_save_handler(SessionHandlerInterface, bool): bool
///
/// When the 1st argument is an object, only 1-2 arguments are allowed.
/// When the 1st argument is callable, at least 6 arguments are required.
#[derive(Default)]
pub struct SessionSetSaveHandlerHook;

impl Provider for SessionSetSaveHandlerHook {
    fn meta() -> &'static ProviderMeta {
        static META: ProviderMeta = ProviderMeta::new(
            "php::session::session_set_save_handler",
            "session_set_save_handler",
            "Validates session_set_save_handler argument combinations.",
        );

        &META
    }
}

impl FunctionCallHook for SessionSetSaveHandlerHook {
    fn after_function_call(&self, call: &FunctionCall<'_>, context: &mut HookContext<'_, '_>) -> HookResult<()> {
        let Expression::Identifier(identifier) = call.function else {
            return Ok(());
        };

        if !identifier.value().eq_ignore_ascii_case("session_set_save_handler") {
            return Ok(());
        }

        let arguments = &call.argument_list.arguments;
        if arguments.is_empty() {
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

        let is_object_form = !first_arg_type.has_callable() && first_arg_type.is_objecty();

        if is_object_form {
            if arguments.len() > 2 {
                let third_arg = arguments.get(2).unwrap();
                let span = match third_arg {
                    Argument::Positional(arg) => arg.value.span(),
                    Argument::Named(arg) => arg.span(),
                };

                context.report(
                    IssueCode::TooManyArguments,
                    Issue::error("Too many arguments provided for function `session_set_save_handler`.")
                        .with_annotation(
                            Annotation::primary(span).with_message("Unexpected argument provided here"),
                        )
                        .with_annotation(
                            Annotation::secondary(call.function.span()).with_message("For this function call"),
                        )
                        .with_note(format!(
                            "When the first argument is a `SessionHandlerInterface`, `session_set_save_handler()` expects at most 2 arguments, but received {}.",
                            arguments.len()
                        ))
                        .with_help("Remove the extra arguments. The object form only accepts the handler and an optional `$register_shutdown` boolean."),
                );
            }
        } else {
            if arguments.len() < 6 {
                context.report(
                    IssueCode::TooFewArguments,
                    Issue::error("Too few arguments provided for function `session_set_save_handler`.")
                        .with_annotation(
                            Annotation::primary(call.argument_list.span())
                                .with_message(format!("Only {} argument(s) provided", arguments.len())),
                        )
                        .with_annotation(
                            Annotation::secondary(call.function.span()).with_message("For this function call"),
                        )
                        .with_note(format!(
                            "The callable form of `session_set_save_handler()` requires at least 6 arguments (`$open`, `$close`, `$read`, `$write`, `$destroy`, `$gc`), but only {} were provided.",
                            arguments.len()
                        ))
                        .with_help("Provide all 6 required callback arguments, or pass a `SessionHandlerInterface` object instead."),
                );
            }
        }

        Ok(())
    }
}
