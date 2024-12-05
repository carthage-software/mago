use fennec_ast::ast::*;
use fennec_reporting::*;
use fennec_span::*;
use fennec_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

#[derive(Clone, Copy, Debug)]
pub struct FunctionRule;

impl Rule for FunctionRule {
    fn get_name(&self) -> &'static str {
        "function"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Help)
    }
}

impl<'a> Walker<LintContext<'a>> for FunctionRule {
    fn walk_in_function<'ast>(&self, function: &'ast Function, context: &mut LintContext) {
        let name = context.lookup(&function.name.value);
        let fqfn = context.lookup_name(&function.name);
        let camel_case = context.option("camel").and_then(|v| v.as_bool()).unwrap_or(false);
        let either_case = context.option("either").and_then(|v| v.as_bool()).unwrap_or(false);

        if either_case {
            if !fennec_casing::is_camel_case(name) && !fennec_casing::is_snake_case(name) {
                context.report(
                    Issue::new(
                        context.level(),
                        format!("function name `{}` should be in either camel case or snake case", name),
                    )
                    .with_annotations([
                        Annotation::primary(function.name.span()),
                        Annotation::secondary(function.span())
                            .with_message(format!("function `{}` is declared here", fqfn)),
                    ])
                    .with_note(format!(
                        "the function name `{}` does not follow either camel case or snake naming convention.",
                        name
                    ))
                    .with_help(format!(
                        "consider renaming it to `{}` or `{}` to adhere to the naming convention.",
                        fennec_casing::to_camel_case(name),
                        fennec_casing::to_snake_case(name)
                    )),
                );
            }

            return;
        }

        if camel_case {
            if !fennec_casing::is_camel_case(name) {
                context.report(
                    Issue::new(context.level(), format!("function name `{}` should be in camel case", name))
                        .with_annotations([
                            Annotation::primary(function.name.span()),
                            Annotation::secondary(function.span())
                                .with_message(format!("function `{}` is declared here", fqfn)),
                        ])
                        .with_note(format!("the function name `{}` does not follow camel naming convention.", name))
                        .with_help(format!(
                            "consider renaming it to `{}` to adhere to the naming convention.",
                            fennec_casing::to_camel_case(name)
                        )),
                );
            }

            return;
        }

        if !fennec_casing::is_snake_case(name) {
            context.report(
                Issue::new(context.level(), format!("function name `{}` should be in snake case", name))
                    .with_annotations([
                        Annotation::primary(function.name.span()),
                        Annotation::secondary(function.span())
                            .with_message(format!("function `{}` is declared here", fqfn)),
                    ])
                    .with_note(format!("the function name `{}` does not follow snake naming convention.", name))
                    .with_help(format!(
                        "consider renaming it to `{}` to adhere to the naming convention.",
                        fennec_casing::to_snake_case(name)
                    )),
            );
        }
    }
}
