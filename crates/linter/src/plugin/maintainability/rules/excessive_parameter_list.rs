use indoc::indoc;
use toml::Value;

use mago_ast::ast::*;
use mago_ast::Node;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
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
}

impl<'a> Walker<LintContext<'a>> for ExcessiveParameterListRule {
    fn walk_in_method(&self, method: &Method, context: &mut LintContext<'a>) {
        check("Method", Node::Method(method), &method.parameter_list, context);
    }

    fn walk_in_function(&self, function: &Function, context: &mut LintContext<'a>) {
        check("Function", Node::Function(function), &function.parameter_list, context);
    }

    fn walk_in_property_hook(&self, property_hook: &PropertyHook, context: &mut LintContext<'a>) {
        if let Some(parameters) = &property_hook.parameters {
            check("Hook", Node::PropertyHook(property_hook), parameters, context);
        }
    }

    fn walk_in_closure(&self, closure: &Closure, context: &mut LintContext<'a>) {
        check("Closure", Node::Closure(closure), &closure.parameter_list, context);
    }

    fn walk_in_arrow_function(&self, arrow_function: &ArrowFunction, context: &mut LintContext<'a>) {
        check("Arrow function", Node::ArrowFunction(arrow_function), &arrow_function.parameter_list, context);
    }
}

#[inline]
fn check(
    kind: &'static str,
    node: Node<'_>,
    parameter_list: &FunctionLikeParameterList,
    context: &mut LintContext<'_>,
) {
    let threshold = context.option(THRESHOLD).and_then(|o| o.as_integer()).unwrap_or(THRESHOLD_DEFAULT) as usize;

    if parameter_list.parameters.len() > threshold {
        let issue = Issue::new(context.level(), format!("{} parameter list is too long.", kind))
            .with_annotation(Annotation::primary(parameter_list.span()).with_message(format!(
                "{} has {} parameters, which exceeds the threshold of {}.",
                kind,
                parameter_list.parameters.len(),
                threshold
            )))
            .with_annotation(Annotation::secondary(node.span()).with_message(format!("{} is defined here", kind)))
            .with_note("Having a large number of parameters can make functions harder to understand and maintain.")
            .with_help("Try reducing the number of parameters, or consider passing an object or a shape instead.");

        context.report(issue);
    }
}
