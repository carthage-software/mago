use fennec_ast::*;
use fennec_reporting::*;
use fennec_span::HasSpan;
use fennec_walker::Walker;

use crate::consts::EXTENSION_FUNCTIONS;
use crate::context::LintContext;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct DisallowedFunctionsRule;

impl DisallowedFunctionsRule {
    fn report_disallowed_function<'ast>(&self, function_call: &'ast FunctionCall, context: &mut LintContext) {
        let Expression::Identifier(identifier) = function_call.function.as_ref() else {
            return;
        };

        let Some(disallowed_functions) = context.option("functions").and_then(|o| o.as_array()) else {
            tracing::trace!("no disallowed functions found in configuration");

            return;
        };

        let function_name = if context.is_name_imported(identifier) {
            context.lookup_name(identifier)
        } else {
            let name = context.interner.lookup(&identifier.value());

            if let Some(stripped) = name.strip_prefix('\\') {
                stripped
            } else {
                name
            }
        };

        if disallowed_functions.iter().any(|f| f.as_str().is_some_and(|f| f.eq(function_name))) {
            let issue = Issue::new(context.level(), format!("disallowed function: `{}`", function_name))
                .with_annotation(Annotation::primary(function_call.span()))
                .with_note(format!("The function `{}` is disallowed by your project configuration.", function_name))
                .with_help("use an alternative function or modify the configuration to allow this function.");

            context.report(issue);
        }
    }

    fn report_disallowed_extension_function<'ast>(&self, function_call: &'ast FunctionCall, context: &mut LintContext) {
        let Expression::Identifier(identifier) = function_call.function.as_ref() else {
            return;
        };

        let Some(disallowed_extensions) = context.option("extensions").and_then(|o| o.as_array()) else {
            tracing::trace!("no disallowed extensions found in configuration");

            return;
        };

        let function_name = context.lookup_function_name(identifier);

        let Some(extension) = EXTENSION_FUNCTIONS.into_iter().find_map(|(extension, function_names)| {
            if function_names.into_iter().any(|f| function_name.eq(*f)) {
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
                format!("disallowed extension function: `{}` from `{}` extension", function_name, extension),
            )
            .with_annotation(Annotation::primary(function_call.span()))
            .with_note(format!(
                "functions from the `{}` extension are disallowed by your project configuration.",
                extension
            ))
            .with_help("use an alternative function or modify the configuration to allow this extension.");

            context.report(issue);
        }
    }
}

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
        self.report_disallowed_function(function_call, context);
        self.report_disallowed_extension_function(function_call, context);
    }
}
