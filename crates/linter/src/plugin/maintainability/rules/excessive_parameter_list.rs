use indoc::indoc;
use toml::Value;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::directive::LintDirective;
use crate::rule::Rule;

const THRESHOLD: &str = "threshold";
const THRESHOLD_DEFAULT: i64 = 5;

#[derive(Clone, Copy, Debug)]
pub struct ExcessiveParameterListRule;

impl Rule for ExcessiveParameterListRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Excessive Parameter List", Level::Error)
            .with_description(indoc! {r#"
                Detects functions, closures, and methods with too many parameters.

                This rule checks the number of parameters in functions, closures, and methods.
                If the number of parameters exceeds a configurable threshold, an issue is reported.
            "#})
            .with_option(RuleOptionDefinition {
                name: THRESHOLD,
                r#type: "integer",
                description: "The maximum allowed number of parameters before triggering an issue.",
                default: Value::Integer(THRESHOLD_DEFAULT),
            })
    }
    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::FunctionLikeParameterList(parameter_list) = node else {
            return LintDirective::default();
        };

        let threshold = context.option(THRESHOLD).and_then(|o| o.as_integer()).unwrap_or(THRESHOLD_DEFAULT) as usize;

        if parameter_list.parameters.len() > threshold {
            let issue = Issue::new(context.level(), "Parameter list is too long.".to_string())
                .with_annotation(Annotation::primary(parameter_list.span()).with_message(format!(
                    "This list has {} parameters, which exceeds the threshold of {}.",
                    parameter_list.parameters.len(),
                    threshold
                )))
                .with_note("Having a large number of parameters can make functions harder to understand and maintain.")
                .with_help("Try reducing the number of parameters, or consider passing an object or a shape instead.");

            context.report(issue);
        }

        LintDirective::Abort
    }
}
