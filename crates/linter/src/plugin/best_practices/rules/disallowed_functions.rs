use indoc::indoc;
use toml::Value;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::consts::EXTENSION_FUNCTIONS;
use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

const FUNCTIONS: &str = "functions";
const EXTENSIONS: &str = "extensions";

#[derive(Clone, Debug)]
pub struct DisallowedFunctionsRule;

impl Rule for DisallowedFunctionsRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Disallowed Functions", Level::Warning)
            .with_description(indoc! {"
                Flags calls to functions that are disallowed via rule configuration. You can specify
                which functions or extensions should be disallowed through the `functions` or `extensions`
                options. This helps enforce organizational coding standards, security restrictions, or
                usage of preferred alternatives.
            "})
            .with_option(RuleOptionDefinition {
                name: FUNCTIONS,
                r#type: "array<string>",
                description: "A list of function names to disallow (case-insensitive).",
                default: Value::Array(vec![]),
            })
            .with_option(RuleOptionDefinition {
                name: EXTENSIONS,
                r#type: "array<string>",
                description: "A list of extension names to disallow (case-insensitive). Any function in these extensions is flagged.",
                default: Value::Array(vec![]),
            })
            .with_example(RuleUsageExample::valid(
                "Calling an allowed function",
                indoc! {"
                    <?php

                    function allowed_function(): void {
                        // ...
                    }

                    allowed_function(); // Not disallowed, no warnings
                "},
            ))
            .with_example(
                RuleUsageExample::valid(
                    "Function bar is not in the disallowed list",
                    indoc! {"
                        <?php

                        function bar() {
                            // ...
                        }

                        bar(); // `bar` is not disallowed, so no warnings
                    "},
                )
                .with_option(FUNCTIONS, Value::Array(vec![Value::String("foo".to_owned())]))
            )
            .with_example(
                RuleUsageExample::invalid(
                    "Calling a function explicitly set as disallowed",
                    indoc! {"
                        <?php

                        function disallowed_func(): void {
                            // ...
                        }

                        disallowed_func(); // Error: 'disallowed_func' is disallowed
                    "},
                )
                .with_option(FUNCTIONS, Value::Array(vec![Value::String("disallowed_func".to_owned())]))
            )
            .with_example(
                RuleUsageExample::invalid(
                    "Calling a function from a disallowed extension",
                    indoc! {"
                        <?php

                        curl_init(); // Error: 'curl_init' is part of the 'curl' extension, which is disallowed
                    "},
                ).with_option(EXTENSIONS, Value::Array(vec![Value::String("curl".to_owned())]))
            )
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::FunctionCall(function_call) = node else { return LintDirective::default() };

        let Expression::Identifier(identifier) = function_call.function.as_ref() else {
            return LintDirective::default();
        };

        let function_name = context.resolve_function_name(identifier);

        // Check if the function is disallowed
        if let Some(disallowed_functions) = context.option(FUNCTIONS).and_then(|o| o.as_array()) {
            if disallowed_functions.iter().any(|f| f.as_str().is_some_and(|f| f.eq_ignore_ascii_case(function_name))) {
                let issue = Issue::new(context.level(), format!("Function `{}` is disallowed.", function_name))
                    .with_annotation(
                        Annotation::primary(function_call.span())
                            .with_message(format!("Function `{}` is called here.`", function_name)),
                    )
                    .with_note(format!("The function `{}` is disallowed by your project configuration.", function_name))
                    .with_help("Use an alternative function or modify the configuration to allow this function.");

                context.report(issue);

                return LintDirective::default();
            }
        }

        // Check if the function is part of a disallowed extension
        if let Some(disallowed_extensions) = context.option(EXTENSIONS).and_then(|o| o.as_array()) {
            let Some(extension) = EXTENSION_FUNCTIONS.into_iter().find_map(|(extension, function_names)| {
                if function_names.iter().any(|f| function_name.eq_ignore_ascii_case(f)) {
                    Some(extension)
                } else {
                    None
                }
            }) else {
                // not an extension function
                return LintDirective::default();
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
        }

        LintDirective::default()
    }
}
