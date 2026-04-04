use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;
use mago_syntax::walker::MutWalker;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches_any;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const MERGE_FUNCTIONS: &[&str] = &["array_merge", "array_merge_recursive"];
const ACCUMULATOR_FUNCTIONS: &[&str] = &["array_unique", "array_values"];

#[derive(Debug, Clone)]
pub struct NoArrayAccumulationInLoopRule {
    meta: &'static RuleMeta,
    cfg: NoArrayAccumulationInLoopConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoArrayAccumulationInLoopConfig {
    pub level: Level,
}

impl Default for NoArrayAccumulationInLoopConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoArrayAccumulationInLoopConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoArrayAccumulationInLoopRule {
    type Config = NoArrayAccumulationInLoopConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Array Accumulation In Loop",
            code: "no-array-accumulation-in-loop",
            description: indoc! {"
                Detects O(n²) array accumulation patterns inside loops.

                Calling `array_merge()`, `array_merge_recursive()`, `array_unique()`, or
                `array_values()` on an accumulator inside a loop copies the entire array on
                every iteration. Similarly, using spread syntax (`[...$result, ...$item]`)
                in a reassignment has the same cost.

                Collect items first and transform once after the loop instead.
            "},
            good_example: indoc! {r"
                <?php

                $chunks = [];
                foreach ($items as $item) {
                    $chunks[] = $item;
                }
                $result = array_merge(...$chunks);
            "},
            bad_example: indoc! {r"
                <?php

                $result = [];
                foreach ($items as $item) {
                    $result = array_merge($result, $item);
                }
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::For, NodeKind::Foreach, NodeKind::While, NodeKind::DoWhile];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let loop_span = node.span();

        let mut collector = AccumulationCollector { findings: Vec::new() };

        match node {
            Node::For(r#for) => match &r#for.body {
                ForBody::Statement(stmt) => collector.walk_statement(stmt, ctx),
                ForBody::ColonDelimited(body) => {
                    for stmt in body.statements.iter() {
                        collector.walk_statement(stmt, ctx);
                    }
                }
            },
            Node::Foreach(foreach) => match &foreach.body {
                ForeachBody::Statement(stmt) => collector.walk_statement(stmt, ctx),
                ForeachBody::ColonDelimited(body) => {
                    for stmt in body.statements.iter() {
                        collector.walk_statement(stmt, ctx);
                    }
                }
            },
            Node::While(r#while) => match &r#while.body {
                WhileBody::Statement(stmt) => collector.walk_statement(stmt, ctx),
                WhileBody::ColonDelimited(body) => {
                    for stmt in body.statements.iter() {
                        collector.walk_statement(stmt, ctx);
                    }
                }
            },
            Node::DoWhile(do_while) => collector.walk_statement(do_while.statement, ctx),
            _ => return,
        };

        for finding in &collector.findings {
            let (message, span, help) = match finding {
                Finding::Merge { function_name, span } => (
                    format!("`{function_name}()` called inside a loop causes O(n²) array copying."),
                    *span,
                    format!("Collect items into an array and call `{function_name}()` once after the loop."),
                ),
                Finding::Transform { function_name, span } => (
                    format!("`{function_name}()` called inside a loop reprocesses the entire array every iteration."),
                    *span,
                    format!("Move `{function_name}()` outside the loop and call it once after accumulation."),
                ),
                Finding::Spread { span } => (
                    "Array spread in reassignment inside a loop causes O(n²) array copying.".to_string(),
                    *span,
                    "Collect items with `$array[] = $item` and merge once after the loop.".to_string(),
                ),
            };

            ctx.collector.report(
                Issue::new(self.cfg.level(), message)
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(span).with_message("expensive operation inside loop"))
                    .with_annotation(Annotation::secondary(loop_span).with_message("inside this loop"))
                    .with_note("This pattern copies the entire accumulator on every iteration, resulting in O(n²) time and memory.")
                    .with_help(help),
            );
        }
    }
}

enum Finding {
    Merge { function_name: &'static str, span: Span },
    Transform { function_name: &'static str, span: Span },
    Spread { span: Span },
}

struct AccumulationCollector {
    findings: Vec<Finding>,
}

impl<'ctx, 'arena> MutWalker<'_, 'arena, LintContext<'ctx, 'arena>> for AccumulationCollector {
    fn walk_in_assignment(&mut self, assignment: &Assignment<'arena>, ctx: &mut LintContext<'ctx, 'arena>) {
        if let Expression::Call(Call::Function(call)) = assignment.rhs {
            if let Some(name) = function_call_matches_any(ctx, call, MERGE_FUNCTIONS) {
                self.findings.push(Finding::Merge { function_name: name, span: call.span() });
                return;
            }

            if let Some(name) = function_call_matches_any(ctx, call, ACCUMULATOR_FUNCTIONS) {
                self.findings.push(Finding::Transform { function_name: name, span: call.span() });
                return;
            }
        }

        // Check: $result = [...$result, ...$item]
        if let Expression::Array(array) = assignment.rhs
            && array.elements.iter().any(|el| el.is_variadic())
        {
            self.findings.push(Finding::Spread { span: assignment.span() });
        }
    }

    // Don't descend into nested loops, they'll be checked separately as their own targets.
    fn walk_for(&mut self, _: &For<'arena>, _: &mut LintContext<'ctx, 'arena>) {}
    fn walk_foreach(&mut self, _: &Foreach<'arena>, _: &mut LintContext<'ctx, 'arena>) {}
    fn walk_while(&mut self, _: &While<'arena>, _: &mut LintContext<'ctx, 'arena>) {}
    fn walk_do_while(&mut self, _: &DoWhile<'arena>, _: &mut LintContext<'ctx, 'arena>) {}

    // Don't descend into nested function scopes.
    fn walk_function(&mut self, _: &Function<'arena>, _: &mut LintContext<'ctx, 'arena>) {}
    fn walk_closure(&mut self, _: &Closure<'arena>, _: &mut LintContext<'ctx, 'arena>) {}
    fn walk_arrow_function(&mut self, _: &ArrowFunction<'arena>, _: &mut LintContext<'ctx, 'arena>) {}
    fn walk_method(&mut self, _: &Method<'arena>, _: &mut LintContext<'ctx, 'arena>) {}
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoArrayAccumulationInLoopRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = array_merge_outside_loop,
        rule = NoArrayAccumulationInLoopRule,
        code = indoc! {r"
            <?php

            $chunks = [];
            foreach ($items as $item) {
                $chunks[] = $item;
            }
            $result = array_merge(...$chunks);
        "}
    }

    test_lint_success! {
        name = simple_push_in_loop,
        rule = NoArrayAccumulationInLoopRule,
        code = indoc! {r"
            <?php

            $result = [];
            foreach ($items as $item) {
                $result[] = $item;
            }
        "}
    }

    test_lint_failure! {
        name = array_merge_in_foreach,
        rule = NoArrayAccumulationInLoopRule,
        code = indoc! {r"
            <?php

            $result = [];
            foreach ($items as $item) {
                $result = array_merge($result, $item);
            }
        "}
    }

    test_lint_failure! {
        name = array_merge_recursive_in_foreach,
        rule = NoArrayAccumulationInLoopRule,
        code = indoc! {r"
            <?php

            $result = [];
            foreach ($items as $item) {
                $result = array_merge_recursive($result, $item);
            }
        "}
    }

    test_lint_failure! {
        name = array_merge_in_for,
        rule = NoArrayAccumulationInLoopRule,
        code = indoc! {r"
            <?php

            $result = [];
            for ($i = 0; $i < 100; $i++) {
                $result = array_merge($result, getItems($i));
            }
        "}
    }

    test_lint_failure! {
        name = array_merge_in_while,
        rule = NoArrayAccumulationInLoopRule,
        code = indoc! {r"
            <?php

            $result = [];
            while ($item = getNext()) {
                $result = array_merge($result, $item);
            }
        "}
    }

    test_lint_failure! {
        name = array_unique_in_loop,
        rule = NoArrayAccumulationInLoopRule,
        code = indoc! {r"
            <?php

            $result = [];
            foreach ($items as $item) {
                $result[] = $item;
                $result = array_unique($result);
            }
        "}
    }

    test_lint_failure! {
        name = array_values_in_loop,
        rule = NoArrayAccumulationInLoopRule,
        code = indoc! {r"
            <?php

            $result = [];
            foreach ($items as $item) {
                $result[] = $item;
                $result = array_values($result);
            }
        "}
    }

    test_lint_failure! {
        name = spread_in_foreach,
        rule = NoArrayAccumulationInLoopRule,
        code = indoc! {r"
            <?php

            $result = [];
            foreach ($items as $item) {
                $result = [...$result, ...$item];
            }
        "}
    }

    test_lint_failure! {
        name = array_merge_uppercase,
        rule = NoArrayAccumulationInLoopRule,
        code = indoc! {r"
            <?php

            $result = [];
            foreach ($items as $item) {
                $result = ARRAY_MERGE($result, $item);
            }
        "}
    }

    test_lint_failure! {
        name = nested_loop_only_inner_reported,
        rule = NoArrayAccumulationInLoopRule,
        count = 1,
        code = indoc! {r"
            <?php

            foreach ($groups as $group) {
                foreach ($group as $item) {
                    $result = array_merge($result, $item);
                }
            }
        "}
    }
}
