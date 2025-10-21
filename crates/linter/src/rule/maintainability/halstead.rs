use mago_span::HasSpan;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct HalsteadRule {
    meta: &'static RuleMeta,
    cfg: HalsteadConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct HalsteadConfig {
    pub level: Level,
    pub volume_threshold: f64,
    pub difficulty_threshold: f64,
    pub effort_threshold: f64,
}

impl Default for HalsteadConfig {
    fn default() -> Self {
        Self { level: Level::Warning, volume_threshold: 1000.0, difficulty_threshold: 12.0, effort_threshold: 5000.0 }
    }
}

impl Config for HalsteadConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for HalsteadRule {
    type Config = HalsteadConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Halstead",
            code: "halstead",
            description: indoc::indoc! {r#"
                Computes Halstead metrics (volume, difficulty, effort) and reports if they exceed configured thresholds.

                Halstead metrics are calculated by counting operators and operands in the analyzed code.
                For more info: https://en.wikipedia.org/wiki/Halstead_complexity_measures
            "#},
            good_example: indoc::indoc! {r#"
                <?php

                function processOrderData($orders) {
                    $filtered = filterValidOrders($orders);
                    $totals = calculateTotals($filtered);
                    return applyDiscounts($totals);
                }

                function filterValidOrders($orders) {
                    return array_filter($orders, fn($order) => $order['status'] === 'valid');
                }

                function calculateTotals($orders) {
                    return array_map(fn($order) => $order['price'] * $order['quantity'], $orders);
                }

                function applyDiscounts($totals) {
                    return array_map(fn($total) => $total > 100 ? $total * 0.9 : $total, $totals);
                }
            "#},
            bad_example: indoc::indoc! {r#"
                <?php

                function processOrderData($orders) {
                    $result = [];
                    $total1 = 0;
                    $total2 = 0;
                    $total3 = 0;
                    $discount1 = 0;
                    $discount2 = 0;
                    $discount3 = 0;
                    $count1 = 0;
                    $count2 = 0;
                    $count3 = 0;
                    $sum1 = 0;
                    $sum2 = 0;
                    $sum3 = 0;

                    for ($i = 0; $i < count($orders); $i++) {
                        $order = $orders[$i];
                        if ($order['status'] === 'pending') {
                            $price = $order['price'];
                            $quantity = $order['quantity'];
                            $subtotal = $price * $quantity;
                            $total1 = $total1 + $subtotal;

                            if ($subtotal > 100) {
                                $discount1 = $subtotal * 0.1;
                                $total1 = $total1 - $discount1;
                                $count1 = $count1 + 1;
                            }

                            if ($subtotal > 200) {
                                $discount2 = $subtotal * 0.15;
                                $total2 = $total2 + $subtotal - $discount2;
                                $count2 = $count2 + 1;
                            }

                            if ($subtotal > 300) {
                                $discount3 = $subtotal * 0.2;
                                $total3 = $total3 + $subtotal - $discount3;
                                $count3 = $count3 + 1;
                            }

                            $sum1 = $sum1 + $price;
                            $sum2 = $sum2 + $quantity;
                            $sum3 = $sum3 + $subtotal;

                            for ($j = 0; $j < $quantity; $j++) {
                                $itemCost = $price / $quantity;
                                $taxRate = 0.08;
                                $tax = $itemCost * $taxRate;
                                $finalCost = $itemCost + $tax;
                                $sum1 = $sum1 + $finalCost;

                                if ($finalCost > 50) {
                                    $extraDiscount = $finalCost * 0.05;
                                    $sum2 = $sum2 + $extraDiscount;
                                }
                            }
                        }
                    }

                    $result['total1'] = $total1;
                    $result['total2'] = $total2;
                    $result['total3'] = $total3;
                    $result['count1'] = $count1;
                    $result['count2'] = $count2;
                    $result['count3'] = $count3;
                    $result['sum1'] = $sum1;
                    $result['sum2'] = $sum2;
                    $result['sum3'] = $sum3;

                    return $result;
                }
            "#},
            category: Category::Maintainability,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[
            NodeKind::PropertyHookConcreteBody,
            NodeKind::Method,
            NodeKind::Function,
            NodeKind::Closure,
            NodeKind::ArrowFunction,
        ];

        TARGETS
    }

    fn build(settings: &RuleSettings<HalsteadConfig>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let kind = match node.kind() {
            NodeKind::PropertyHookConcreteBody => "Hook",
            NodeKind::Method => "Method",
            NodeKind::Function => "Function",
            NodeKind::Closure => "Closure",
            NodeKind::ArrowFunction => "Arrow function",
            _ => return,
        };

        let halstead = gather_and_compute_halstead(node);

        if halstead.volume > self.cfg.volume_threshold {
            ctx.collector.report(
                Issue::new(self.cfg.level, format!("{kind} has a high halstead volume"))
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(node.span()).with_message(format!(
                        "{} has a halstead volume of {}, which exceeds the threshold of {}.",
                        kind, halstead.volume, self.cfg.volume_threshold
                    )))
                    .with_note("Halstead volume estimates the code's overall size/complexity."),
            );
        }

        if halstead.difficulty > self.cfg.difficulty_threshold {
            ctx.collector.report(
                Issue::new(self.cfg.level, format!("{kind} has a high halstead difficulty"))
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(node.span()).with_message(format!(
                        "{} has a halstead difficulty of {}, which exceeds the threshold of {}.",
                        kind, halstead.difficulty, self.cfg.difficulty_threshold
                    )))
                    .with_note("Halstead difficulty reflects how hard the code is to write or understand."),
            );
        }

        if halstead.effort > self.cfg.effort_threshold {
            ctx.collector.report(
                Issue::new(self.cfg.level, format!("{kind} has a high halstead effort"))
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(node.span()).with_message(format!(
                        "{} has a halstead effort of {}, which exceeds the threshold of {}.",
                        kind, halstead.effort, self.cfg.effort_threshold
                    )))
                    .with_note("Halstead effort estimates the mental effort required to develop/maintain the code."),
            );
        }
    }
}

#[derive(Debug)]
struct HalsteadMetrics {
    pub volume: f64,     // V
    pub difficulty: f64, // D
    pub effort: f64,     // E
}

#[inline]
fn gather_and_compute_halstead<'ast, 'arena>(node: Node<'ast, 'arena>) -> HalsteadMetrics {
    let (operators, operands) = gather_operators_and_operands(node);

    compute_halstead_metrics(&operators, &operands)
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct Operator(NodeKind);

#[derive(Debug, Hash, Eq, PartialEq)]
struct Operand<'arena>(&'arena str);

#[inline]
fn gather_operators_and_operands<'ast, 'arena>(node: Node<'ast, 'arena>) -> (Vec<Operator>, Vec<Operand<'arena>>) {
    let mut operators = Vec::new();
    let mut operands = Vec::new();

    fn recurse<'ast, 'arena>(n: Node<'ast, 'arena>, ops: &mut Vec<Operator>, rands: &mut Vec<Operand<'arena>>) {
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
fn categorize_node<'arena>(node: Node<'_, 'arena>, operators: &mut Vec<Operator>, operands: &mut Vec<Operand<'arena>>) {
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
            operands.push(Operand(literal.raw));
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
