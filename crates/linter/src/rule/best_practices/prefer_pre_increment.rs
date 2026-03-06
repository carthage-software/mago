use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::UnaryPostfix;
use mago_syntax::ast::UnaryPostfixOperator;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferPreIncrementRule {
    meta: &'static RuleMeta,
    cfg: PreferPreIncrementConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PreferPreIncrementConfig {
    pub level: Level,
}

impl Default for PreferPreIncrementConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for PreferPreIncrementConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferPreIncrementRule {
    type Config = PreferPreIncrementConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Pre-Increment",
            code: "prefer-pre-increment",
            description: indoc! {"
                Enforces the use of pre-increment (`++$i`) and pre-decrement (`--$i`) over
                post-increment (`$i++`) and post-decrement (`$i--`).

                Pre-increment is marginally more efficient and is the convention used by
                the Symfony coding standards.
            "},
            good_example: indoc! {r"
                <?php

                ++$i;
                --$count;
            "},
            bad_example: indoc! {r"
                <?php

                $i++;
                $count--;
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        // Target ExpressionStatement (standalone `$i++;`) and For (increment expressions)
        // to only flag post-increment where the return value is discarded.
        // This avoids false positives on `$a = $b++` where semantics would change.
        const TARGETS: &[NodeKind] = &[NodeKind::ExpressionStatement, NodeKind::For];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        match node {
            Node::ExpressionStatement(expr_stmt) => {
                // Standalone statement: `$i++;` — safe to convert, value is discarded
                if let Expression::UnaryPostfix(unary_postfix) = expr_stmt.expression {
                    self.report(ctx, unary_postfix);
                }
            }
            Node::For(for_stmt) => {
                // For-loop increment expressions: `for (...; ...; $i++)` — value is discarded
                for expr in for_stmt.increments.iter() {
                    if let Expression::UnaryPostfix(unary_postfix) = expr {
                        self.report(ctx, unary_postfix);
                    }
                }
            }
            _ => {}
        }
    }
}

impl PreferPreIncrementRule {
    fn report<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, unary_postfix: &UnaryPostfix<'arena>) {
        let (message, fix_op) = match unary_postfix.operator {
            UnaryPostfixOperator::PostIncrement(_) => {
                ("Use pre-increment `++$var` instead of post-increment `$var++`", "++")
            }
            UnaryPostfixOperator::PostDecrement(_) => {
                ("Use pre-decrement `--$var` instead of post-decrement `$var--`", "--")
            }
        };

        let issue = Issue::new(self.cfg.level(), message)
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(unary_postfix.operator.span()).with_message("Post-increment/decrement operator"),
            )
            .with_help("Move the operator before the variable: `++$var` instead of `$var++`");

        let source_code = ctx.source_file.contents.as_ref();
        ctx.collector.propose(issue, |edits| {
            let operand_span = unary_postfix.operand.span();
            let operand_text = &source_code[operand_span.start_offset() as usize..operand_span.end_offset() as usize];

            let full_span = unary_postfix.span();
            let replacement = format!("{}{}", fix_op, operand_text);
            edits.push(TextEdit::replace(full_span, &replacement));
        });
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::PreferPreIncrementRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = pre_increment_is_ok,
        rule = PreferPreIncrementRule,
        code = indoc! {r"
            <?php

            ++$i;
            --$count;
        "}
    }

    test_lint_failure! {
        name = post_increment_is_bad,
        rule = PreferPreIncrementRule,
        code = indoc! {r"
            <?php

            $i++;
        "}
    }

    test_lint_failure! {
        name = post_decrement_is_bad,
        rule = PreferPreIncrementRule,
        code = indoc! {r"
            <?php

            $count--;
        "}
    }

    test_lint_failure! {
        name = post_increment_in_for_loop_is_bad,
        rule = PreferPreIncrementRule,
        code = indoc! {r"
            <?php

            for ($i = 0; $i < $count; $i++) {
            }
        "}
    }

    test_lint_success! {
        name = post_increment_in_assignment_is_ok,
        rule = PreferPreIncrementRule,
        code = indoc! {r"
            <?php

            $a = $b++;
        "}
    }

    test_lint_success! {
        name = post_increment_in_function_arg_is_ok,
        rule = PreferPreIncrementRule,
        code = indoc! {r"
            <?php

            foo($b++);
        "}
    }

    test_lint_success! {
        name = post_increment_in_array_index_is_ok,
        rule = PreferPreIncrementRule,
        code = indoc! {r"
            <?php

            $a[$b++];
        "}
    }

    test_lint_success! {
        name = post_increment_in_return_is_ok,
        rule = PreferPreIncrementRule,
        code = indoc! {r"
            <?php

            function foo() {
                return $i++;
            }
        "}
    }
}
