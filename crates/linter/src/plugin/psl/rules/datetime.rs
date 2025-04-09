use std::sync::LazyLock;

use ahash::HashMap;
use indoc::indoc;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::plugin::psl::rules::utils::format_replacements;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct DateTimeRule;

impl Rule for DateTimeRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("DateTime", Level::Warning)
            .with_description(indoc! {"
                This rule enforces the usage of Psl DateTime classes and functions over their PHP counterparts.

                Psl DateTime classes and functions are preferred because they are type-safe and provide more consistent behavior.
            "})
            .with_example(RuleUsageExample::valid(
                "Using `Psl\\DateTime\\DateTime` instead of `DateTime`.",
                indoc! {r#"
                    <?php

                    $dateTime = new Psl\DateTime\DateTime();
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Using `Psl\\DateTime\\DateTime::now` instead of `new DateTime()`.",
                indoc! {r#"
                    <?php

                    $now = Psl\DateTime\DateTime::now();
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `DateTime`.",
                indoc! {r#"
                    <?php

                    $dateTime = new DateTime();
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `strtotime`.",
                indoc! {r#"
                    <?php

                    $timestamp = strtotime('now');
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let identifier = match node {
            Node::FunctionCall(function_call) => {
                let Expression::Identifier(identifier) = function_call.function.as_ref() else {
                    return LintDirective::default();
                };

                let function_name = context.resolve_function_name(identifier).to_lowercase();

                if let Some(replacements) = (*DATETIME_FUNCTION_REPLACEMENTS).get(function_name.as_str()) {
                    context.report(
                        Issue::new(context.level(), "Use the Psl DateTime function instead of the PHP counterpart.")
                            .with_annotation(Annotation::primary(identifier.span()).with_message("This is a PHP DateTime function."))
                            .with_note("Psl DateTime classes and functions are preferred because they are type-safe and provide more consistent behavior.")
                            .with_help(format!("Use {} instead.", format_replacements(replacements))),
                    );
                }

                return LintDirective::default();
            }
            Node::Instantiation(instantiation) => {
                let Expression::Identifier(identifier) = instantiation.class.as_ref() else {
                    return LintDirective::default();
                };

                identifier
            }
            Node::Call(Call::StaticMethod(static_method_call)) => {
                let Expression::Identifier(identifier) = static_method_call.class.as_ref() else {
                    return LintDirective::default();
                };

                identifier
            }
            _ => return LintDirective::default(),
        };

        let class_name = context.lookup_name(identifier).to_lowercase();
        if let Some(replacements) = DATETIME_CLASS_REPLACEMENTS.get(class_name.as_str()) {
            context.report(
                Issue::new(context.level(), "Use the Psl DateTime class instead of the PHP counterpart.")
                    .with_annotation(Annotation::primary(identifier.span()).with_message("This is a PHP DateTime class."))
                    .with_note("Psl DateTime classes and functions are preferred because they are type-safe and provide more consistent behavior.")
                    .with_help(format!("Use {} instead.", format_replacements(replacements))),
            );
        }

        LintDirective::default()
    }
}

static DATETIME_FUNCTION_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("time", vec!["Psl\\DateTime\\Timestamp::now", "Psl\\DateTime\\DateTime::now"]),
        ("microtime", vec!["Psl\\DateTime\\Timestamp::now", "Psl\\DateTime\\DateTime::now"]),
        ("hrtime", vec!["Psl\\DateTime\\Timestamp::monotonic"]),
        (
            "strtotime",
            vec![
                "Psl\\DateTime\\Timestamp::parse",
                "Psl\\DateTime\\Timestamp::fromString",
                "Psl\\DateTime\\DateTime::parse",
                "Psl\\DateTime\\DateTime::fromString",
            ],
        ),
        ("date_default_timezone_get", vec!["Psl\\DateTime\\Timezone::default"]),
    ])
});

static DATETIME_CLASS_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("datetime", vec!["Psl\\DateTime\\DateTime"]),
        ("datetimeimmutable", vec!["Psl\\DateTime\\DateTime"]),
        ("datetimezone", vec!["Psl\\DateTime\\Timezone"]),
        ("dateinterval", vec!["Psl\\DateTime\\Duration"]),
        ("intldateformatter", vec!["Psl\\DateTime\\DateTime"]),
        ("intltimezone", vec!["Psl\\DateTime\\Timezone"]),
        ("intltimezone", vec!["Psl\\DateTime\\Timezone"]),
    ])
});
