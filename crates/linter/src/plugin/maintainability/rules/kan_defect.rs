use indoc::indoc;
use toml::Value;

use mago_ast::*;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::directive::LintDirective;
use crate::rule::Rule;

const THRESHOLD: &str = "threshold";
const THRESHOLD_DEFAULT: f64 = 1.6;

#[derive(Clone, Copy, Debug)]
pub struct KanDefectRule;

impl Rule for KanDefectRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Kan Defect", Level::Error)
            .with_description(indoc! {r#"
                Detects classes, traits, interfaces, functions, and closures with high kan defect.

                The "Kan Defect" metric is a heuristic for estimating defect proneness in a class or similar structure.
                It counts control-flow statements (`while`, `do`, `foreach`, `if`, and `switch`) and sums them using a
                formula loosely based on the work of Stephen H. Kan.

                References:
                  - https://github.com/phpmetrics/PhpMetrics/blob/c43217cd7783bbd54d0b8c1dd43f697bc36ef79d/src/Hal/Metric/Class_/Complexity/KanDefectVisitor.php
                  - https://phpmetrics.org/
            "#})
            .with_option(RuleOptionDefinition {
                name: THRESHOLD,
                r#type: "float",
                description: "The maximum allowed kan defect score before triggering an issue.",
                default: Value::Float(THRESHOLD_DEFAULT ),
            })
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let kind = match node {
            Node::Class(_) => "Class",
            Node::Trait(_) => "Trait",
            Node::AnonymousClass(_) => "Class",
            Node::Enum(_) => "Enum",
            Node::Interface(_) => "Interface",
            Node::Function(_) => "Function",
            Node::Closure(_) => "Closure",
            _ => return LintDirective::default(),
        };

        let threshold = context
            .option(THRESHOLD)
            .and_then(|o| if o.is_integer() { o.as_integer().map(|i| i as f64) } else { o.as_float() })
            .unwrap_or(THRESHOLD_DEFAULT);

        let kan_defect = get_kan_defect_of_node(node);
        if kan_defect > threshold {
            let issue = Issue::new(context.level(), format!("{kind} has a high kan defect score ({kan_defect})."))
                .with_annotation(Annotation::primary(node.span()).with_message(format!(
                    "{kind} has a kan defect score of {kan_defect}, which exceeds the threshold of {threshold}.",
                )))
                .with_note("Kan defect is a heuristic used by phpmetrics to estimate defect-proneness based on control-flow statements.")
                .with_help("Try reducing the number of loops, switch statements, or if statements.")
                .with_help("You can also consider splitting large units of code into smaller, more focused units.");

            context.report(issue);
        }

        LintDirective::Prune
    }
}

/// Returns the *Kan Defect* value for a given AST node by counting
/// the number of select statements, loop statements, and `if` statements,
/// then applying the **Kan Defect** formula.
#[inline]
fn get_kan_defect_of_node(node: Node<'_>) -> f64 {
    let (select_count, while_count, if_count) = collect_defect_factors(node);
    calculate_kan_defect(select_count, while_count, if_count)
}

/// Computes the final *Kan Defect* value given the counts of select
/// statements (`switch`/`match`), loop statements (`do…while`, `while`,
/// `foreach`), and `if` statements.
///
/// This formula is taken from the phpmetrics “Kan Defect” metric:
///
/// ```text
/// defect = 0.15
///        + 0.23 * (number of loops)
///        + 0.22 * (number of selects)
///        + 0.07 * (number of ifs)
/// ```
///
/// Note that these coefficients (0.15, 0.23, 0.22, 0.07) are an *approximation*
/// and are not part of a standard software metric outside phpmetrics.
///
/// See: https://github.com/phpmetrics/PhpMetrics/blob/c43217cd7783bbd54d0b8c1dd43f697bc36ef79d/src/Hal/Metric/Class_/Complexity/KanDefectVisitor.php#L60C13-L60C76
#[inline]
fn calculate_kan_defect(select: usize, r#while: usize, r#if: usize) -> f64 {
    let select = select as f64;
    let r#while = r#while as f64;
    let r#if = r#if as f64;

    0.15 + 0.23 * r#while + 0.22 * select + 0.07 * r#if
}

/// Recursively traverses the given AST node, counting:
///
/// 1. **Select statements** (i.e., `switch` or `match`)
/// 2. **Loop statements** (i.e., `do…while`, `while`, `foreach`)
/// 3. **If statements** (`if`)
///
/// Returns a tuple `(select_count, while_count, if_count)` representing
/// how many of each type of statement appear under the given node and all
/// its descendants.
#[inline]
fn collect_defect_factors(node: Node<'_>) -> (usize, usize, usize) {
    let mut select_count = 0;
    let mut while_count = 0;
    let mut if_count = 0;

    // Recurse through child nodes
    for child in node.children() {
        let (child_select, child_while, child_if) = collect_defect_factors(child);

        select_count += child_select;
        while_count += child_while;
        if_count += child_if;
    }

    // Check the current node's type
    match node {
        Node::Switch(_) | Node::Match(_) => {
            select_count += 1;
        }
        Node::DoWhile(_) | Node::While(_) | Node::Foreach(_) => {
            while_count += 1;
        }
        Node::If(_) => {
            if_count += 1;
        }
        _ => (),
    }

    (select_count, while_count, if_count)
}
