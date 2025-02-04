use indoc::indoc;
use toml::Value;

use mago_ast::*;
use mago_interner::StringIdentifier;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::directive::LintDirective;
use crate::rule::Rule;

/// volume (V)
pub const VOLUME_THRESHOLD: &str = "volume_threshold";
pub const VOLUME_THRESHOLD_DEFAULT: f64 = 1000.0;

/// difficulty (D)
pub const DIFFICULTY_THRESHOLD: &str = "difficulty_threshold";
pub const DIFFICULTY_THRESHOLD_DEFAULT: f64 = 12.0;

/// effort (E)
pub const EFFORT_THRESHOLD: &str = "effort_threshold";
pub const EFFORT_THRESHOLD_DEFAULT: f64 = 5000.0;

/// A rule that computes Halstead metrics and compares them against user-defined thresholds.
#[derive(Clone, Copy, Debug)]
pub struct HalsteadRule;

impl Rule for HalsteadRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Halstead", Level::Warning)
            .with_description(indoc! {r#"
                This rule computes several Halstead metrics (volume, difficulty, effort, time)
                and checks whether each exceeds a configurable threshold. If any threshold is
                exceeded, an error is reported.

                Halstead metrics are calculated by counting operators and operands in the
                analyzed code. For more details, see: https://en.wikipedia.org/wiki/Halstead_complexity_measures
            "#})
            .with_option(RuleOptionDefinition {
                name: VOLUME_THRESHOLD,
                r#type: "float",
                description: "Maximum allowed halstead volume (V).",
                default: Value::Float(VOLUME_THRESHOLD_DEFAULT),
            })
            .with_option(RuleOptionDefinition {
                name: DIFFICULTY_THRESHOLD,
                r#type: "float",
                description: "Maximum allowed halstead difficulty (D).",
                default: Value::Float(DIFFICULTY_THRESHOLD_DEFAULT),
            })
            .with_option(RuleOptionDefinition {
                name: EFFORT_THRESHOLD,
                r#type: "float",
                description: "Maximum allowed halstead effort (E).",
                default: Value::Float(EFFORT_THRESHOLD_DEFAULT),
            })
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let kind = match node {
            Node::PropertyHookConcreteBody(_) => "Hook",
            Node::Method(_) => "Method",
            Node::Function(_) => "Function",
            Node::Closure(_) => "Closure",
            Node::ArrowFunction(_) => "Arrow function",
            _ => return LintDirective::Continue,
        };

        // Gather operators/operands, compute Halstead
        let halstead = gather_and_compute_halstead(node);

        // Compare results with thresholds
        check_against_thresholds(kind, &halstead, &node, context);

        LintDirective::Prune
    }
}

/// Compares each metric to the user-configured thresholds. If any metric
/// exceeds its threshold, we report an error.
#[inline]
fn check_against_thresholds(
    kind: &'static str,
    halstead: &HalsteadMetrics,
    node: &Node<'_>,
    context: &mut LintContext<'_>,
) {
    let volume_threshold = context
        .option(VOLUME_THRESHOLD)
        .and_then(|o| if o.is_integer() { o.as_integer().map(|i| i as f64) } else { o.as_float() })
        .unwrap_or(VOLUME_THRESHOLD_DEFAULT);

    let difficulty_threshold = context
        .option(DIFFICULTY_THRESHOLD)
        .and_then(|o| if o.is_integer() { o.as_integer().map(|i| i as f64) } else { o.as_float() })
        .unwrap_or(DIFFICULTY_THRESHOLD_DEFAULT);

    let effort_threshold = context
        .option(EFFORT_THRESHOLD)
        .and_then(|o| if o.is_integer() { o.as_integer().map(|i| i as f64) } else { o.as_float() })
        .unwrap_or(EFFORT_THRESHOLD_DEFAULT);

    // VOLUME (V)
    if halstead.volume > volume_threshold {
        let issue = Issue::new(context.level(), format!("{kind} has a high halstead volume"))
            .with_annotation(Annotation::primary(node.span()).with_message(format!(
                "{} has a halstead volume of {}, which exceeds the threshold of {}.",
                kind, halstead.volume, volume_threshold
            )))
            .with_note("Halstead volume estimates the code's overall size/complexity.");
        context.report(issue);
    }

    // DIFFICULTY (D)
    if halstead.difficulty > difficulty_threshold {
        let issue = Issue::new(context.level(), format!("{kind} has a high halstead difficulty"))
            .with_annotation(Annotation::primary(node.span()).with_message(format!(
                "{} has a halstead difficulty of {}, which exceeds the threshold of {}.",
                kind, halstead.difficulty, difficulty_threshold
            )))
            .with_note("Halstead difficulty reflects how hard the code is to write or understand.");
        context.report(issue);
    }

    // EFFORT (E)
    if halstead.effort > effort_threshold {
        let issue = Issue::new(context.level(), format!("{kind} has a high halstead effort"))
            .with_annotation(Annotation::primary(node.span()).with_message(format!(
                "{} has a halstead effort of {}, which exceeds the threshold of {}.",
                kind, halstead.effort, effort_threshold
            )))
            .with_note("Halstead effort estimates the mental effort required to develop/maintain the code.");
        context.report(issue);
    }
}

#[derive(Debug)]
struct HalsteadMetrics {
    pub volume: f64,     // V
    pub difficulty: f64, // D
    pub effort: f64,     // E
}

#[inline]
fn gather_and_compute_halstead(node: Node<'_>) -> HalsteadMetrics {
    let (operators, operands) = gather_operators_and_operands(node);

    compute_halstead_metrics(&operators, &operands)
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct Operator(NodeKind);

#[derive(Debug, Hash, Eq, PartialEq)]
struct Operand(StringIdentifier);

#[inline]
fn gather_operators_and_operands(node: Node<'_>) -> (Vec<Operator>, Vec<Operand>) {
    let mut operators = Vec::new();
    let mut operands = Vec::new();

    fn recurse(n: Node<'_>, ops: &mut Vec<Operator>, rands: &mut Vec<Operand>) {
        if n.is_declaration() {
            return;
        }

        for child in n.children() {
            recurse(child, ops, rands);
        }

        categorize_node(n, ops, rands);
    }

    for child in node.children() {
        recurse(child, &mut operators, &mut operands);
    }

    (operators, operands)
}

/// Check if the node is considered an operator or operand in Halstead terms
/// and record a textual representation.
#[inline]
fn categorize_node(node: Node<'_>, operators: &mut Vec<Operator>, operands: &mut Vec<Operand>) {
    match node {
        Node::Binary(_)
        | Node::Assignment(_)
        | Node::If(_)
        | Node::IfStatementBodyElseIfClause(_)
        | Node::IfColonDelimitedBodyElseIfClause(_)
        | Node::For(_)
        | Node::Switch(_)
        | Node::TryCatchClause(_)
        | Node::Return(_)
        | Node::While(_)
        | Node::DoWhile(_) => {
            operators.push(Operator(node.kind()));
        }
        Node::UnaryPrefix(unary) if unary.operator.is_cast() => {
            operators.push(Operator(node.kind()));
        }
        Node::DirectVariable(variable) => {
            operands.push(Operand(variable.name));
        }
        Node::LiteralString(literal) => {
            operands.push(Operand(literal.value));
        }
        Node::LiteralInteger(literal) => {
            operands.push(Operand(literal.raw));
        }
        Node::LiteralFloat(literal) => {
            operands.push(Operand(literal.raw));
        }
        _ => (),
    }
}

/// Computes the Halstead metrics from the given operators and operands.
///
/// **Important**: if `n2 == 0` or `N2 == 0`, we set all metrics to 0
/// (mirroring the original phpmetrics approach).
#[inline]
fn compute_halstead_metrics(operators: &[Operator], operands: &[Operand]) -> HalsteadMetrics {
    use std::collections::HashSet;

    let unique_ops: HashSet<_> = operators.iter().collect();
    let unique_operands: HashSet<_> = operands.iter().collect();

    let n1 = unique_ops.len();
    let n2 = unique_operands.len();
    let total_n1 = operators.len();
    let total_n2 = operands.len();

    if n2 == 0 || total_n2 == 0 {
        return HalsteadMetrics { volume: 0.0, difficulty: 0.0, effort: 0.0 };
    }

    let n1_f = n1 as f64;
    let n2_f = n2 as f64;
    let total_n1_f = total_n1 as f64;
    let total_n2_f = total_n2 as f64;

    let n = n1_f + n2_f;
    let total_n = total_n1_f + total_n2_f;

    let volume = if n > 0.0 { total_n * n.log2() } else { 0.0 };
    let difficulty = (n1_f / 2.0) * (total_n2_f / n2_f.max(1.0));
    let effort = volume * difficulty;

    HalsteadMetrics { volume: round2(volume), difficulty: round2(difficulty), effort: round2(effort) }
}

/// Utility to round a floating-point number to two decimal places.
#[inline]
fn round2(val: f64) -> f64 {
    (val * 100.0).round() / 100.0
}
