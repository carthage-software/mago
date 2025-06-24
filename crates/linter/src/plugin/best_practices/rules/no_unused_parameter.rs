use indoc::indoc;

use mago_codex::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
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
        RuleDefinition::enabled("No Unused Parameter", Level::Note)
            .with_description(indoc! {"
            Detects parameters that are declared but never used within a function, method, or closure.
            Unused parameters are a sign of dead code and can be safely removed to improve code clarity.
        "})
            .with_example(RuleUsageExample::valid(
                "Function with used parameters",
                indoc! {r#"
                <?php

                declare(strict_types=1);

                class Container {
                    private array $data = [];

                    public function set(string $key, string $value, ?string $default = null): void {
                        $this->data[$key] = $value ?? $default;
                    }
                }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Function with unused parameter",
                indoc! {r#"
                <?php

                declare(strict_types=1);

                class Container {
                    private array $data = [];

                    public function set(string $key, string $value, ?string $default = null): void {
                        $this->data[$key] = $value;
                    }
                }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let (metadata, members) = match node {
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
                let name = context.resolved_names.get(&class.name);
                let Some(metadata) = get_class(context.codebase, context.interner, name) else {
                    return LintDirective::default();
                };

                (metadata, class.members.iter())
            }
            Node::Enum(r#enum) => {
                let name = context.resolved_names.get(&r#enum.name);
                let Some(metadata) = get_enum(context.codebase, context.interner, name) else {
                    return LintDirective::default();
                };

                (metadata, r#enum.members.iter())
            }
            Node::Trait(r#trait) => {
                let name = context.resolved_names.get(&r#trait.name);
                let Some(metadata) = get_trait(context.codebase, context.interner, name) else {
                    return LintDirective::default();
                };

                (metadata, r#trait.members.iter())
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

            if is_method_overriding(context.codebase, context.interner, &metadata.name, &method.name.value) {
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

    let issue = Issue::new(context.level(), format!("Parameter `{parameter_name}` is never used."))
        .with_annotations([
            Annotation::primary(parameter.span()).with_message(format!("Parameter `{parameter_name}` is declared here.")),
            Annotation::secondary(function_like.span()),
        ])
        .with_note(format!("This parameter is declared but not used within the {kind}."))
        .with_help("Consider prefixing the parameter with an underscore (`_`) to indicate that it is intentionally unused, or remove it if it is not needed.");

    context.propose(issue, |plan| {
        plan.insert(
            parameter.variable.span().start.offset + 1, // skip the leading `$`
            "_",
            SafetyClassification::PotentiallyUnsafe,
        );
    });
}
