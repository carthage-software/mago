use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::Expression;
use mago_syntax::cst::ForBody;
use mago_syntax::cst::ForeachBody;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::Statement;
use mago_syntax::cst::WhileBody;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::early_exit::EarlyExitPattern;
use crate::rule::utils::early_exit::check_early_exit;
use crate::rule::utils::early_exit::extract_single_if_from_statement;
use crate::rule::utils::early_exit::extract_single_if_from_statements;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferEarlyContinueRule {
    meta: &'static RuleMeta,
    cfg: PreferEarlyContinueConfig,
}

#[derive(Debug, Clone, Copy, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct PreferEarlyContinueConfig {
    pub level: Level,
    pub max_allowed_statements: usize,
}

impl Default for PreferEarlyContinueConfig {
    fn default() -> Self {
        Self { level: Level::Help, max_allowed_statements: 0 }
    }
}

impl Config for PreferEarlyContinueConfig {
    fn level(&self) -> Level {
        self.level
    }
}

#[allow(clippy::unwrap_used)]
impl LintRule for PreferEarlyContinueRule {
    type Config = PreferEarlyContinueConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Early Continue",
            code: "prefer-early-continue",
            description: indoc! {"
                Suggests using early continue pattern when a loop body contains only a single if statement.

                This improves code readability by reducing nesting and making the control flow more explicit.
            "},
            good_example: indoc! {r"
                <?php

                for ($i = 0; $i < 10; $i++) {
                    if (!$condition) {
                        continue;
                    }
                    doSomething();
                }
            "},
            bad_example: indoc! {r"
                <?php

                for ($i = 0; $i < 10; $i++) {
                    if ($condition) {
                        doSomething();
                    }
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

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let (if_statement, loop_span) = match node {
            Node::For(for_loop) => {
                let body = match &for_loop.body {
                    ForBody::Statement(stmt) => extract_single_if_from_statement(stmt),
                    ForBody::ColonDelimited(block) => extract_single_if_from_statements(block.statements.as_slice()),
                };
                (body, for_loop.span())
            }
            Node::Foreach(foreach_loop) => {
                let body = match &foreach_loop.body {
                    ForeachBody::Statement(stmt) => extract_single_if_from_statement(stmt),
                    ForeachBody::ColonDelimited(block) => {
                        extract_single_if_from_statements(block.statements.as_slice())
                    }
                };
                (body, foreach_loop.span())
            }
            Node::While(while_loop) => {
                let body = match &while_loop.body {
                    WhileBody::Statement(stmt) => extract_single_if_from_statement(stmt),
                    WhileBody::ColonDelimited(block) => extract_single_if_from_statements(block.statements.as_slice()),
                };
                (body, while_loop.span())
            }
            Node::DoWhile(do_while) => (extract_single_if_from_statement(do_while.statement), do_while.span()),
            _ => return,
        };

        let Some(if_stmt) = if_statement else { return };

        check_early_exit(
            ctx,
            self.cfg.level(),
            if_stmt,
            loop_span,
            self.cfg.max_allowed_statements,
            &EarlyExitPattern {
                keyword: "continue",
                code: self.meta.code,
                title: "Consider using early continue pattern to reduce nesting.",
                primary_message: "This if statement wraps the entire loop body",
                secondary_message: "The loop can benefit from early continue to improve readability",
                help: "Invert the condition and use `continue` to exit early, then place the main logic outside the if block.",
                is_early_exit_statement,
            },
        );
    }
}

fn is_early_exit_statement(stmt: &Statement) -> bool {
    match stmt {
        Statement::Continue(_) | Statement::Break(_) | Statement::Return(_) => true,
        Statement::Expression(expr) => matches!(expr.expression, Expression::Throw(_)),
        Statement::Block(block) => block.statements.len() == 1 && is_early_exit_statement(&block.statements.nodes[0]),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::PreferEarlyContinueRule;
    use crate::test_lint_fix;

    test_lint_fix! {
        name = fix_without_comments_keeps_compact_shape,
        rule = PreferEarlyContinueRule,
        code = indoc! {r"
            <?php

            foreach ($nodes as $node) {
                if (is_array($node)) {
                    process($node);
                }
            }
        "},
        fixed = indoc! {r"
            <?php

            foreach ($nodes as $node) {
                if (!(is_array($node))) { continue; }

            process($node);
            }
        "}
    }

    // Regression for issue #1946: comments inside the if body must survive
    // the rewrite. The result may be poorly indented; `--fmt` handles that.
    test_lint_fix! {
        name = fix_preserves_leading_comment,
        rule = PreferEarlyContinueRule,
        code = indoc! {r"
            <?php

            foreach ($nodes as $node) {
                if (is_array($node)) {
                    // keep only array nodes
                    process($node);
                }
            }
        "},
        fixed = indoc! {r"
            <?php

            foreach ($nodes as $node) {
                if (!(is_array($node))) { continue; }

                    // keep only array nodes
                    process($node);
            }
        "}
    }

    test_lint_fix! {
        name = fix_preserves_trailing_comment,
        rule = PreferEarlyContinueRule,
        code = indoc! {r"
            <?php

            foreach ($nodes as $node) {
                if (is_array($node)) {
                    process($node);
                    // trailing note
                }
            }
        "},
        fixed = concat!(
            "<?php\n",
            "\n",
            "foreach ($nodes as $node) {\n",
            "    if (!(is_array($node))) { continue; }\n",
            "\n",
            "process($node);\n",
            "        // trailing note\n",
            "    \n",
            "}\n",
        )
    }

    test_lint_fix! {
        name = fix_preserves_comments_in_colon_delimited_body,
        rule = PreferEarlyContinueRule,
        code = indoc! {r"
            <?php

            foreach ($nodes as $node):
                if (is_array($node)):
                    // leading note
                    process($node);
                    // trailing note
                endif;
            endforeach;
        "},
        fixed = concat!(
            "<?php\n",
            "\n",
            "foreach ($nodes as $node):\n",
            "    if (!(is_array($node))){ continue; }\n",
            "\n",
            "        // leading note\n",
            "        process($node);\n",
            "        // trailing note\n",
            "    \n",
            "endforeach;\n",
        )
    }

    test_lint_fix! {
        name = fix_preserves_comment_before_non_block_statement,
        rule = PreferEarlyContinueRule,
        code = indoc! {r"
            <?php

            foreach ($nodes as $node) {
                if (is_array($node)) /* keep */ process($node);
            }
        "},
        fixed = indoc! {r"
            <?php

            foreach ($nodes as $node) {
                if (!(is_array($node))) { continue; }
             /* keep */ process($node);
            }
        "}
    }
}
