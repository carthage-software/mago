use mago_ast::*;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::consts::EXTENSION_FUNCTIONS;
use crate::context::LintContext;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct DisallowedFunctionsRule;

impl Rule for DisallowedFunctionsRule {
    fn get_name(&self) -> &'static str {
        "disallowed-functions"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Warning)
    }
}

impl<'a> Walker<LintContext<'a>> for DisallowedFunctionsRule {
    fn walk_in_function_call<'ast>(&self, function_call: &'ast FunctionCall, context: &mut LintContext<'a>) {
        let Expression::Identifier(identifier) = function_call.function.as_ref() else {
            return;
        };

        let function_name = context.resolve_function_name(identifier);

        // Check if the function is disallowed
        if let Some(disallowed_functions) = context.option("functions").and_then(|o| o.as_array()) {
            if disallowed_functions.iter().any(|f| f.as_str().is_some_and(|f| f.eq(function_name))) {
                let issue = Issue::new(context.level(), format!("Function `{}` is disallowed.", function_name))
                    .with_annotation(
                        Annotation::primary(function_call.span())
                            .with_message(format!("Function `{}` is called here.`", function_name)),
                    )
                    .with_note(format!("The function `{}` is disallowed by your project configuration.", function_name))
                    .with_help("Use an alternative function or modify the configuration to allow this function.");

                context.report(issue);

                return;
            }
        } else {
            tracing::trace!("No disallowed functions found in configuration.");
        };

        // Check if the function is part of a disallowed extension
        if let Some(disallowed_extensions) = context.option("extensions").and_then(|o| o.as_array()) {
            let Some(extension) = EXTENSION_FUNCTIONS.into_iter().find_map(|(extension, function_names)| {
                if function_names.iter().any(|f| function_name.eq(*f)) {
                    Some(extension)
                } else {
                    None
                }
            }) else {
                // not an extension function

                return;
            };

            if disallowed_extensions.iter().any(|e| e.as_str().is_some_and(|e| e.eq(extension))) {
                let issue = Issue::new(
                    context.level(),
                    format!("Function `{}` from the `{}` extension is disallowed.", function_name, extension),
                )
                .with_annotation(
                    Annotation::primary(function_call.span())
                        .with_message(format!("Function `{}` is called here.", function_name)),
                )
                .with_note(format!(
                    "Functions from the `{}` extension are disallowed by your project configuration.",
                    extension
                ))
                .with_help("Use an alternative function or modify the configuration to allow this extension.");

                context.report(issue);
            }
        } else {
            tracing::trace!("No disallowed extensions found in configuration.");
        }
    }
}
