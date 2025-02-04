use indoc::indoc;
use toml::Value;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct ArraySyntaxRule;

const SYNTAX: &str = "syntax";
const SYNTAX_LONG: &str = "long";
const SYNTAX_SHORT: &str = "short";
const SYNTAX_DEFAULT: &str = SYNTAX_SHORT;

impl Rule for ArraySyntaxRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Array Syntax", Level::Note)
            .with_description(indoc! {"
            Suggests using the short array syntax `[..]` instead of the long array syntax `array(..)`,
            or vice versa, depending on the configuration. The short array syntax is more concise and
            is the preferred way to define arrays in PHP.
        "})
            .with_option(RuleOptionDefinition {
                name: SYNTAX,
                r#type: "string",
                description: "The array syntax to enforce. Can be either `short` or `long`.",
                default: Value::String(SYNTAX_DEFAULT.to_string()),
            })
            .with_example({
                RuleUsageExample::valid(
                    "Using short array syntax by default",
                    indoc! {"
                        <?php

                        // By default, `syntax` is 'short', so this snippet is valid:
                        $arr = [1, 2, 3];
                    "},
                )
            })
            .with_example(
                RuleUsageExample::valid(
                    "Using long array syntax when configured",
                    indoc! {r#"
                        <?php

                        // If we set `syntax = "long"`, then array(...) is correct:
                        $arr = array(1, 2, 3);
                    "#},
                )
                .with_option(SYNTAX, Value::String(SYNTAX_LONG.to_string())),
            )
            .with_example({
                RuleUsageExample::invalid(
                    "Using long array syntax when `syntax=short` is the default",
                    indoc! {r#"
                        <?php

                        // By default, 'short' is enforced, so array(...) triggers a warning:
                        $arr = array(1, 2, 3);
                    "#},
                )
            })
            .with_example(
                RuleUsageExample::invalid(
                    "Using short array syntax when `syntax=long` is configured",
                    indoc! {r#"
                        <?php

                        // If we set `syntax = "long"`, [..] is disallowed:
                        $arr = [1, 2, 3];
                    "#},
                )
                .with_option(SYNTAX, Value::String(SYNTAX_LONG.to_string())),
            )
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        match node {
            Node::LegacyArray(arr) => {
                let preferred_syntax = context.option(SYNTAX).and_then(|o| o.as_str()).unwrap_or(SYNTAX_DEFAULT);
                if !preferred_syntax.eq_ignore_ascii_case(SYNTAX_SHORT) {
                    return LintDirective::default();
                }

                let issue = Issue::new(context.level(), "Short array syntax `[..]` is preferred over `array(..)`.")
                    .with_annotation(
                        Annotation::primary(arr.span())
                            .with_message("This array uses the long array syntax `array(..)`."),
                    )
                    .with_help("Use the short array syntax `[..]` instead");

                context.report_with_fix(issue, |plan| {
                    plan.replace(arr.array.span.join(arr.left_parenthesis).to_range(), "[", SafetyClassification::Safe);
                    plan.replace(arr.right_parenthesis.to_range(), "]", SafetyClassification::Safe);
                });
            }
            Node::Array(arr) => {
                let preferred_syntax = context.option(SYNTAX).and_then(|o| o.as_str()).unwrap_or(SYNTAX_DEFAULT);
                if !preferred_syntax.eq_ignore_ascii_case(SYNTAX_LONG) {
                    return LintDirective::default();
                }

                let issue = Issue::new(context.level(), "Long array syntax `array(..)` is preferred over `[..]`.")
                    .with_annotation(
                        Annotation::primary(arr.span()).with_message("This array uses the short array syntax `[..]`."),
                    )
                    .with_help("Use the long array syntax `array(..)` instead");

                context.report_with_fix(issue, |plan| {
                    plan.replace(arr.left_bracket.to_range(), "array(", SafetyClassification::Safe);
                    plan.replace(arr.right_bracket.to_range(), ")", SafetyClassification::Safe)
                });
            }
            _ => {}
        }

        LintDirective::default()
    }
}
