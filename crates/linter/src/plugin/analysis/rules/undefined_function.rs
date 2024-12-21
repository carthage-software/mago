use mago_ast::*;
use mago_reporting::*;
use mago_span::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct UndefinedFunctionRule;

impl Rule for UndefinedFunctionRule {
    fn get_name(&self) -> &'static str {
        "undefined-function"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Error)
    }
}

impl<'a> Walker<LintContext<'a>> for UndefinedFunctionRule {
    fn walk_in_function_call(&self, function_call: &FunctionCall, context: &mut LintContext<'a>) {
        let Expression::Identifier(identifier) = function_call.function.as_ref() else {
            return;
        };

        let function_name = context.resolve_function_name(identifier);
        let function_name_id = context.interner.intern(function_name);
        let function_name_lower = context.interner.lowered(&function_name_id);
        if context.codebase.function_exists(&function_name_lower) {
            return;
        }

        let issue = Issue::error(format!("Call to undefined function `{}`.", function_name))
            .with_annotation(
                Annotation::primary(identifier.span())
                    .with_message(format!("Function `{}` does not exist.", function_name)),
            )
            .with_help(format!("Ensure the function `{}` is defined or imported before calling it.", function_name));

        context.report(issue);
    }
}
