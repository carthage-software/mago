use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Expression;
use mago_syntax::ast::ForBody;
use mago_syntax::ast::ForeachBody;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Statement;
use mago_syntax::ast::WhileBody;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoArrayMergeInLoopRule {
    meta: &'static RuleMeta,
    cfg: NoArrayMergeInLoopConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoArrayMergeInLoopConfig {
    pub level: Level,
}

impl Default for NoArrayMergeInLoopConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoArrayMergeInLoopConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoArrayMergeInLoopRule {
    type Config = NoArrayMergeInLoopConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No array_merge In Loop",
            code: "no-array-merge-in-loop",
            description: indoc! {"
                Flags `array_merge()` calls inside `foreach`, `for`, `while`, and `do-while` loops.
                Calling `array_merge()` in a loop causes quadratic time complexity because it copies
                the entire array on each iteration. Use spread operator or collect values and merge once.
            "},
            good_example: indoc! {r"
                <?php

                $collected = [];
                foreach ($items as $item) {
                    $collected[] = $item;
                }
                $result = array_merge($base, $collected);
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
        let statements: &[Statement<'_>] = match node {
            Node::For(for_loop) => match &for_loop.body {
                ForBody::Statement(stmt) => return self.check_statement(ctx, stmt),
                ForBody::ColonDelimited(body) => body.statements.as_slice(),
            },
            Node::Foreach(foreach) => match &foreach.body {
                ForeachBody::Statement(stmt) => return self.check_statement(ctx, stmt),
                ForeachBody::ColonDelimited(body) => body.statements.as_slice(),
            },
            Node::While(while_loop) => match &while_loop.body {
                WhileBody::Statement(stmt) => return self.check_statement(ctx, stmt),
                WhileBody::ColonDelimited(body) => body.statements.as_slice(),
            },
            Node::DoWhile(do_while) => return self.check_statement(ctx, do_while.statement),
            _ => return,
        };

        for stmt in statements {
            self.check_statement(ctx, stmt);
        }
    }
}

impl NoArrayMergeInLoopRule {
    fn check_statement<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, stmt: &Statement<'arena>) {
        match stmt {
            Statement::Expression(expr_stmt) => {
                self.check_expression(ctx, expr_stmt.expression);
            }
            Statement::Block(block) => {
                for s in block.statements.as_slice() {
                    self.check_statement(ctx, s);
                }
            }
            Statement::Return(ret) => {
                if let Some(val) = &ret.value {
                    self.check_expression(ctx, val);
                }
            }
            _ => {}
        }
    }

    fn check_expression<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, expr: &Expression<'arena>) {
        match expr {
            Expression::Call(call) => {
                if let mago_syntax::ast::Call::Function(func_call) = call {
                    if let Expression::Identifier(ident) = func_call.function {
                        let name = ident.value();
                        if name.eq_ignore_ascii_case("array_merge") {
                            ctx.collector.report(
                                Issue::new(
                                    self.cfg.level(),
                                    "`array_merge()` inside a loop has poor performance.",
                                )
                                .with_code(self.meta.code)
                                .with_annotation(
                                    Annotation::primary(func_call.span())
                                        .with_message("This `array_merge()` is called on every iteration"),
                                )
                                .with_help(
                                    "Collect values into an array first, then merge once after the loop. \
                                     Or use the spread operator: `$result = [...$result, ...$item]`.",
                                ),
                            );
                        }
                    }
                    // Also check arguments for nested array_merge calls
                    for arg in func_call.argument_list.arguments.as_slice() {
                        self.check_expression(ctx, arg.value());
                    }
                }
            }
            Expression::Assignment(assignment) => {
                self.check_expression(ctx, assignment.rhs);
            }
            Expression::Parenthesized(paren) => {
                self.check_expression(ctx, paren.expression);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = array_merge_outside_loop,
        rule = NoArrayMergeInLoopRule,
        code = r#"
            <?php

            $result = array_merge($a, $b);
        "#
    }

    test_lint_failure! {
        name = array_merge_in_foreach,
        rule = NoArrayMergeInLoopRule,
        code = r#"
            <?php

            $result = [];
            foreach ($items as $item) {
                $result = array_merge($result, $item);
            }
        "#
    }

    test_lint_failure! {
        name = array_merge_in_for,
        rule = NoArrayMergeInLoopRule,
        code = r#"
            <?php

            $result = [];
            for ($i = 0; $i < 10; $i++) {
                $result = array_merge($result, getItems($i));
            }
        "#
    }
}
