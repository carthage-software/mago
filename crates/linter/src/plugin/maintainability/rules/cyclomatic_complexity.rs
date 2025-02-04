use indoc::indoc;
use toml::Value;

use mago_ast::*;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::directive::LintDirective;
use crate::plugin::maintainability::rules::utils::is_method_setter_or_getter;
use crate::rule::Rule;

const THRESHOLD: &str = "threshold";
const THRESHOLD_DEFAULT: usize = 15;

#[derive(Clone, Copy, Debug)]
pub struct CyclomaticComplexityRule;

impl Rule for CyclomaticComplexityRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Cyclomatic Complexity", Level::Error)
            .with_description(indoc! {r#"
                This rule checks the cyclomatic complexity of classes, traits, enums, interfaces, functions, and closures.

                Cyclomatic complexity is a software metric used to indicate the complexity of a program.

                It is a quantitative measure of the number of linearly independent paths through a program's source code. It is a measure of the number of decisions a program has to make.
            "#})
            .with_option(RuleOptionDefinition {
                name: THRESHOLD,
                r#type: "integer",
                description: "The maximum cyclomatic complexity allowed for a class-like structure.",
                default: Value::Integer(THRESHOLD_DEFAULT as i64),
            })
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        match node {
            Node::Class(node) => check_class_like("Class", node, node.members.as_slice(), context),
            Node::Trait(node) => check_class_like("Trait", node, node.members.as_slice(), context),
            Node::AnonymousClass(node) => check_class_like("Class", node, node.members.as_slice(), context),
            Node::Enum(node) => check_class_like("Enum", node, node.members.as_slice(), context),
            Node::Interface(node) => check_class_like("Interface", node, node.members.as_slice(), context),
            Node::Function(node) => check_function_like("Function", node, &node.body, context),
            Node::Closure(node) => check_function_like("Closure", node, &node.body, context),
            _ => {
                return LintDirective::Continue;
            }
        }

        LintDirective::Prune
    }
}

#[inline]
fn check_class_like(
    kind: &'static str,
    class_like: impl HasSpan,
    members: &[ClassLikeMember],
    context: &mut LintContext<'_>,
) {
    let threshold = context.option(THRESHOLD).and_then(|o| o.as_integer()).unwrap_or(THRESHOLD_DEFAULT as i64);

    let class_like_cyclomatic_complexity = get_cyclomatic_complexity_of_class_like_members(members, context);
    if class_like_cyclomatic_complexity > threshold as usize {
        let issue = Issue::new(context.level(), format!("{kind} has high complexity.")).with_annotation(
            Annotation::primary(class_like.span()).with_message(format!(
                "{} has a cyclomatic complexity of {}, which exceeds the threshold of {}.",
                kind, class_like_cyclomatic_complexity, threshold
            )),
        );

        context.report(issue);
    }
}

#[inline]
fn check_function_like(kind: &'static str, function_like: impl HasSpan, body: &Block, context: &mut LintContext<'_>) {
    let threshold = context.option(THRESHOLD).and_then(|o| o.as_integer()).unwrap_or(THRESHOLD_DEFAULT as i64);

    let block_cyclomatic_complexity = get_cyclomatic_complexity_of_node(Node::Block(body));
    if block_cyclomatic_complexity > threshold as usize {
        let issue = Issue::new(context.level(), format!("{kind} has high complexity.")).with_annotation(
            Annotation::primary(function_like.span()).with_message(format!(
                "{} has a cyclomatic complexity of {}, which exceeds the threshold of {}.",
                kind, block_cyclomatic_complexity, threshold
            )),
        );

        context.report(issue);
    }
}

#[inline]
fn get_cyclomatic_complexity_of_class_like_members(
    class_like_members: &[ClassLikeMember],
    context: &LintContext<'_>,
) -> usize {
    let mut cyclomatic_complexity = 0;
    for member in class_like_members {
        let ClassLikeMember::Method(method) = member else {
            continue;
        };

        let Some(method_cyclomatic_complexity) = get_cyclomatic_complexity_of_method(method, context) else {
            continue;
        };

        cyclomatic_complexity += method_cyclomatic_complexity - 1;
    }

    cyclomatic_complexity
}

#[inline]
fn get_cyclomatic_complexity_of_method(method: &Method, context: &LintContext<'_>) -> Option<usize> {
    if is_method_setter_or_getter(method, context) {
        return None;
    }

    Some(if method.is_abstract() { 1 } else { get_cyclomatic_complexity_of_node(Node::Method(method)) + 1 })
}

#[inline]
fn get_cyclomatic_complexity_of_node(node: Node<'_>) -> usize {
    let mut number = 0;

    for child in node.children() {
        number += get_cyclomatic_complexity_of_node(child);
    }

    match node {
        Node::If(_)
        | Node::IfStatementBodyElseIfClause(_)
        | Node::IfColonDelimitedBodyElseIfClause(_)
        | Node::For(_)
        | Node::Foreach(_)
        | Node::While(_)
        | Node::DoWhile(_)
        | Node::TryCatchClause(_)
        | Node::Conditional(_) => number += 1,
        Node::Binary(operation) => match operation.operator {
            operator if operator.is_logical() || operator.is_null_coalesce() => number += 1,
            BinaryOperator::Spaceship(_) => number += 2,
            _ => (),
        },
        Node::SwitchCase(case) if case.is_default() => {
            number += 1;
        }
        _ => (),
    }

    number
}
