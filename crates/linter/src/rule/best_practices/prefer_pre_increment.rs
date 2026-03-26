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
use crate::integration::Integration;
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
            requirements: RuleRequirements::Integration(Integration::Symfony),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::ExpressionStatement, NodeKind::For];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        match node {
            Node::ExpressionStatement(expr_stmt) => {
                if let Expression::UnaryPostfix(unary_postfix) = expr_stmt.expression {
                    self.report(ctx, unary_postfix);
                }
            }
            Node::For(for_stmt) => {
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
        let (message, annotation_msg, fix_op) = match unary_postfix.operator {
            UnaryPostfixOperator::PostIncrement(_) => {
                ("Use pre-increment `++$var` instead of post-increment `$var++`", "Post-increment operator", "++")
            }
            UnaryPostfixOperator::PostDecrement(_) => {
                ("Use pre-decrement `--$var` instead of post-decrement `$var--`", "Post-decrement operator", "--")
            }
        };

        let issue = Issue::new(self.cfg.level(), message)
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(unary_postfix.operator.span()).with_message(annotation_msg))
            .with_note("Pre-increment/decrement is preferred by the Symfony coding standard as it avoids creating an unused temporary copy")
            .with_help("Move the operator before the variable: `++$var` instead of `$var++`");

        ctx.collector.propose(issue, |edits| {
            let operator_span = unary_postfix.operator.span();
            let operand_span = unary_postfix.operand.span();

            edits.push(TextEdit::delete(operator_span));
            edits.push(TextEdit::insert(operand_span.start_offset(), fix_op));
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
