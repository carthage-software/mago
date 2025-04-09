use indoc::indoc;

use mago_fixer::SafetyClassification;
use mago_php_version::PHPVersion;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Copy, Debug)]
pub struct ReturnByReferenceFromVoidFunctionRule;

impl Rule for ReturnByReferenceFromVoidFunctionRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Return By Reference From Void Function", Level::Warning)
            .with_minimum_supported_php_version(PHPVersion::PHP82)
            .with_description(indoc! {"
                Detects functions, methods, closures, arrow functions, and set property hooks that return by reference from a void function.
                Such functions are considered deprecated; returning by reference from a void function is deprecated since PHP 8.0.
            "})
            .with_example(RuleUsageExample::valid(
                "Returning by reference from a non-void function",
                indoc! {r#"
                    <?php

                    function &foo(): string
                    {
                        // ...
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Returning by reference from a void function",
                indoc! {r#"
                    <?php

                    function &foo(): void
                    {
                        // ...
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Returning by reference from a void method",
                indoc! {r#"
                    <?php

                    class MyClass
                    {
                        public function &foo(): void
                        {
                            // ...
                        }
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Returning by reference from a void closure",
                indoc! {r#"
                    <?php

                    $fun = function &(): void {
                        // ...
                    };
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Returning by reference from a void arrow function",
                indoc! {r#"
                    <?php

                    $fun = fn &(): void => throw new Exception();
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Returning by reference from a property set hook",
                indoc! {r#"
                    <?php

                    class MyClass
                    {
                        public string $property {
                            &set(string $value) {
                                // ...
                            }
                        }
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        match node {
            Node::Function(function) => {
                let Some(amperstand) = function.ampersand.as_ref() else {
                    return LintDirective::default();
                };

                let Some(return_type) = function.return_type_hint.as_ref() else {
                    return LintDirective::default();
                };

                if !matches!(return_type.hint, Hint::Void(_)) {
                    return LintDirective::default();
                }

                report(context, "function", function.span(), amperstand, false);
            }
            Node::Method(method) => {
                let Some(amperstand) = method.ampersand.as_ref() else {
                    return LintDirective::default();
                };

                let Some(return_type) = method.return_type_hint.as_ref() else {
                    return LintDirective::default();
                };

                if !matches!(return_type.hint, Hint::Void(_)) {
                    return LintDirective::default();
                }

                report(context, "method", method.span(), amperstand, false);
            }
            Node::Closure(closure) => {
                let Some(amperstand) = closure.ampersand.as_ref() else {
                    return LintDirective::default();
                };

                let Some(return_type) = closure.return_type_hint.as_ref() else {
                    return LintDirective::default();
                };

                if !matches!(return_type.hint, Hint::Void(_)) {
                    return LintDirective::default();
                }

                report(context, "closure", closure.span(), amperstand, false);
            }
            Node::ArrowFunction(arrow_function) => {
                let Some(amperstand) = arrow_function.ampersand.as_ref() else {
                    return LintDirective::default();
                };

                let Some(return_type) = arrow_function.return_type_hint.as_ref() else {
                    return LintDirective::default();
                };

                if !matches!(return_type.hint, Hint::Void(_)) {
                    return LintDirective::default();
                }

                report(context, "arrow function", arrow_function.span(), amperstand, false);
            }
            Node::PropertyHook(property_hook) => {
                let name = context.lookup(&property_hook.name.value);
                if "set" != name {
                    return LintDirective::default();
                }

                let Some(amperstand) = property_hook.ampersand.as_ref() else {
                    return LintDirective::default();
                };

                report(context, "set property hook", property_hook.span(), amperstand, true);
            }
            _ => (),
        }

        LintDirective::default()
    }
}

fn report(context: &mut LintContext<'_>, kind: &'static str, span: Span, ampersand: &Span, is_set_hook: bool) {
    let message = if !is_set_hook {
        format!("Returning by reference from a void {} is deprecated since PHP 8.0.", kind)
    } else {
        "Returning by reference from a set property hook is deprecated since PHP 8.0".to_string()
    };

    let issue = Issue::new(context.level(), message)
        .with_annotation(
            Annotation::primary(*ampersand)
                .with_message(format!("The `&` indicates that the {} returns by reference.", kind)),
        )
        .with_annotation(Annotation::secondary(span))
        .with_help("Consider removing the `&` to comply with PHP 8.0 standards and avoid future issues.".to_string());

    context.propose(issue, |plan| {
        plan.delete(ampersand.to_range(), SafetyClassification::Safe);
    });
}
