use indoc::indoc;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::directive::LintDirective;
use crate::plugin::best_practices::rules::utils::expression_potentially_contains_function_call;
use crate::plugin::best_practices::rules::utils::get_foreign_variable_names;
use crate::plugin::best_practices::rules::utils::is_variable_used_in_expression;
use crate::plugin::best_practices::rules::utils::potentially_contains_function_call;
use crate::rule::Rule;

const FUNC_GET_ARGS: &str = "func_get_args";

#[derive(Clone, Debug)]
pub struct NoUnusedParameterRule;

impl Rule for NoUnusedParameterRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Unused Parameter", Level::Note).with_description(indoc! {"
            Detects parameters that are declared but never used within a function, method, or closure.
            Unused parameters are a sign of dead code and can be safely removed to improve code clarity.
        "})
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let (reflection, members) = match node {
            Node::Function(function) => {
                if potentially_contains_function_call(&function.body, FUNC_GET_ARGS, context) {
                    // `func_get_args` is potentially used, so we can't determine if the parameters are unused
                    // in this case
                    return LintDirective::default();
                }

                let foreign_variables = get_foreign_variable_names(&function.body, context);
                for parameter in function.parameter_list.parameters.iter() {
                    if foreign_variables.contains(&parameter.variable.name) {
                        continue;
                    }

                    check_parameter(parameter, function, context, "function");
                }

                return LintDirective::default();
            }
            Node::Closure(closure) => {
                if potentially_contains_function_call(&closure.body, FUNC_GET_ARGS, context) {
                    // `func_get_args` is potentially used, so we can't determine if the parameters are unused
                    // in this case
                    return LintDirective::default();
                }

                let foreign_variables = get_foreign_variable_names(&closure.body, context);

                for parameter in closure.parameter_list.parameters.iter() {
                    if foreign_variables.contains(&parameter.variable.name) {
                        continue;
                    }

                    check_parameter(parameter, closure, context, "closure");
                }

                return LintDirective::default();
            }
            Node::ArrowFunction(arrow_function) => {
                if expression_potentially_contains_function_call(&arrow_function.expression, FUNC_GET_ARGS, context) {
                    // `func_get_args` is potentially used, so we can't determine if the parameters are unused
                    // in this case
                    return LintDirective::default();
                }

                for parameter in arrow_function.parameter_list.parameters.iter() {
                    if !is_variable_used_in_expression(&arrow_function.expression, context, parameter.variable.name) {
                        check_parameter(parameter, arrow_function, context, "arrow function");
                    }
                }

                return LintDirective::default();
            }
            Node::Class(class) => {
                let name = context.semantics.names.get(&class.name);
                let Some(reflection) = context.codebase.get_class(context.interner, name) else {
                    return LintDirective::default();
                };

                (reflection, class.members.iter())
            }
            Node::Enum(r#enum) => {
                let name = context.semantics.names.get(&r#enum.name);
                let Some(reflection) = context.codebase.get_enum(context.interner, name) else {
                    return LintDirective::default();
                };

                (reflection, r#enum.members.iter())
            }
            Node::Trait(r#trait) => {
                let name = context.semantics.names.get(&r#trait.name);
                let Some(reflection) = context.codebase.get_trait(context.interner, name) else {
                    return LintDirective::default();
                };

                (reflection, r#trait.members.iter())
            }
            _ => return LintDirective::default(),
        };

        for member in members {
            let ClassLikeMember::Method(method) = member else {
                continue;
            };

            let MethodBody::Concrete(block) = &method.body else {
                continue;
            };

            let Some(method_reflection) = reflection.get_method(&method.name.value) else {
                continue;
            };

            if method_reflection.is_overriding {
                // This method is overriding a method from a parent class.
                continue;
            }

            if potentially_contains_function_call(block, FUNC_GET_ARGS, context) {
                // `func_get_args` is potentially used, so we can't determine if the parameters are unused
                // in this case
                continue;
            }

            let foreign_variables = get_foreign_variable_names(block, context);

            for parameter in method.parameter_list.parameters.iter() {
                // Skip promoted properties
                if parameter.is_promoted_property() {
                    continue;
                }

                if foreign_variables.contains(&parameter.variable.name) {
                    continue;
                }

                check_parameter(parameter, method, context, "method");
            }
        }

        LintDirective::default()
    }
}

fn check_parameter(
    parameter: &FunctionLikeParameter,
    function_like: &impl HasSpan,
    context: &mut LintContext,
    kind: &'static str,
) {
    if parameter.ampersand.is_some() {
        return;
    }

    let parameter_name = context.interner.lookup(&parameter.variable.name);
    if parameter_name.starts_with("$_") {
        return;
    }

    let issue = Issue::new(context.level(), format!("Parameter `{}` is never used.", parameter_name))
        .with_annotations([
            Annotation::primary(parameter.span()).with_message(format!("Parameter `{}` is declared here.", parameter_name)),
            Annotation::secondary(function_like.span()),
        ])
        .with_note(format!("This parameter is declared but not used within the {}.", kind))
        .with_help("Consider prefixing the parameter with an underscore (`_`) to indicate that it is intentionally unused, or remove it if it is not needed.");

    context.report_with_fix(issue, |plan| {
        plan.insert(
            parameter.variable.span().start.offset + 1, // skip the leading `$`
            "_",
            SafetyClassification::PotentiallyUnsafe,
        );
    });
}
