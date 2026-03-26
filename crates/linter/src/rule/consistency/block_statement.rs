use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::ForBody;
use mago_syntax::ast::ForeachBody;
use mago_syntax::ast::IfBody;
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
pub struct BlockStatementRule {
    meta: &'static RuleMeta,
    cfg: BlockStatementConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct BlockStatementConfig {
    pub level: Level,
}

impl Default for BlockStatementConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for BlockStatementConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for BlockStatementRule {
    type Config = BlockStatementConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Block Statement",
            code: "block-statement",
            description: indoc! {"
                Enforces that `if`, `else`, `for`, `foreach`, `while`, `do-while` statements always use a block
                statement body (`{ ... }`) even if they contain only a single statement.

                This improves readability and prevents potential errors when adding new statements.
            "},
            good_example: indoc! {r#"
                <?php

                if (true) {
                    echo "Hello";
                }

                for ($i = 0; $i < 10; $i++) {
                    echo $i;
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                if (true)
                    echo "Hello";

                for ($i = 0; $i < 10; $i++)
                    echo $i;
            "#},
            category: Category::Consistency,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] =
            &[NodeKind::If, NodeKind::For, NodeKind::Foreach, NodeKind::While, NodeKind::DoWhile];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let mut report = |construct_name: &str, construct_span: Span, body_span: Span| {
            let issue = Issue::new(
                self.cfg.level,
                format!("`{construct_name}` statement should use a block body."),
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(body_span).with_message("This statement is not wrapped in a block"),
            )
            .with_annotation(
                Annotation::secondary(construct_span)
                    .with_message(format!("`{construct_name}` statement is defined here")),
            )
            .with_note(
                "Always using block statements `{...}` improves readability and prevents bugs when adding more lines of code.",
            )
            .with_help(format!("Add curly braces `{{ .. }}` around the body of the `{construct_name}` statement."));

            ctx.collector.report(issue);
        };

        match node {
            Node::If(if_stmt) => {
                let IfBody::Statement(body) = &if_stmt.body else {
                    return;
                };

                if !matches!(body.statement, Statement::Block(_)) {
                    report("if", if_stmt.r#if.span(), body.statement.span());
                }

                for else_if_clause in &body.else_if_clauses {
                    if !matches!(else_if_clause.statement, Statement::Block(_)) {
                        report("else if", else_if_clause.elseif.span(), else_if_clause.statement.span());
                    }
                }

                if let Some(else_clause) = &body.else_clause
                    && !matches!(else_clause.statement, Statement::Block(_) | Statement::If(_))
                {
                    report("else", else_clause.r#else.span(), else_clause.statement.span());
                }
            }
            Node::For(r#for) => {
                let ForBody::Statement(statement) = &r#for.body else {
                    return;
                };

                if !matches!(statement, Statement::Block(_)) {
                    report("for", r#for.r#for.span(), statement.span());
                }
            }
            Node::Foreach(r#foreach) => {
                let ForeachBody::Statement(statement) = &r#foreach.body else {
                    return;
                };

                if !matches!(statement, Statement::Block(_)) {
                    report("foreach", r#foreach.r#foreach.span(), statement.span());
                }
            }
            Node::While(r#while) => {
                let WhileBody::Statement(statement) = &r#while.body else {
                    return;
                };

                if !matches!(statement, Statement::Block(_)) {
                    report("while", r#while.r#while.span(), statement.span());
                }
            }
            Node::DoWhile(do_while) if !matches!(do_while.statement, Statement::Block(_)) => {
                report("do-while", do_while.r#do.span(), do_while.statement.span());
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
    use indoc::indoc;

    test_lint_success! {
        name = if_with_block,
        rule = BlockStatementRule,
        code = indoc! {r#"
            <?php

            if ($foo) {
                echo "Hello";
            }
        "#}
    }

    test_lint_success! {
        name = if_else_with_blocks,
        rule = BlockStatementRule,
        code = indoc! {r#"
            <?php

            if ($foo) {
                echo "Hello";
            } else {
                echo "World";
            }
        "#}
    }

    test_lint_success! {
        name = if_elseif_else_with_blocks,
        rule = BlockStatementRule,
        code = indoc! {r#"
            <?php

            if ($foo) {
                echo "Hello";
            } elseif ($bar) {
                echo "World";
            } else {
                echo "!";
            }
        "#}
    }

    test_lint_success! {
        name = if_else_if_with_blocks,
        rule = BlockStatementRule,
        code = indoc! {r#"
            <?php

            if ($foo) {
                echo "Hello";
            } else if ($bar) {
                echo "World";
            } else {
                echo "!";
            }
        "#}
    }

    test_lint_success! {
        name = if_else_if_chain_with_blocks,
        rule = BlockStatementRule,
        code = indoc! {r#"
            <?php

            if ($a) {
                echo "a";
            } else if ($b) {
                echo "b";
            } else if ($c) {
                echo "c";
            } else {
                echo "d";
            }
        "#}
    }

    test_lint_success! {
        name = for_with_block,
        rule = BlockStatementRule,
        code = indoc! {r#"
            <?php

            for ($i = 0; $i < 10; $i++) {
                echo $i;
            }
        "#}
    }

    test_lint_success! {
        name = foreach_with_block,
        rule = BlockStatementRule,
        code = indoc! {r#"
            <?php

            foreach ($items as $item) {
                echo $item;
            }
        "#}
    }

    test_lint_success! {
        name = while_with_block,
        rule = BlockStatementRule,
        code = indoc! {r#"
            <?php

            while ($foo) {
                echo "Hello";
            }
        "#}
    }

    test_lint_success! {
        name = do_while_with_block,
        rule = BlockStatementRule,
        code = indoc! {r#"
            <?php

            do {
                echo "Hello";
            } while ($foo);
        "#}
    }

    test_lint_success! {
        name = colon_delimited_if,
        rule = BlockStatementRule,
        code = indoc! {r#"
            <?php

            if ($foo):
                echo "Hello";
            endif;
        "#}
    }

    test_lint_success! {
        name = colon_delimited_for,
        rule = BlockStatementRule,
        code = indoc! {r#"
            <?php

            for ($i = 0; $i < 10; $i++):
                echo $i;
            endfor;
        "#}
    }

    test_lint_failure! {
        name = if_without_block,
        rule = BlockStatementRule,
        count = 1,
        code = indoc! {r#"
            <?php

            if ($foo)
                echo "Hello";
        "#}
    }

    test_lint_failure! {
        name = if_with_block_else_without_block,
        rule = BlockStatementRule,
        count = 1,
        code = indoc! {r#"
            <?php

            if ($foo) {
                echo "Hello";
            } else
                echo "World";
        "#}
    }

    test_lint_failure! {
        name = if_without_block_else_without_block,
        rule = BlockStatementRule,
        count = 2,
        code = indoc! {r#"
            <?php

            if ($foo)
                echo "Hello";
            else
                echo "World";
        "#}
    }

    test_lint_failure! {
        name = elseif_without_block,
        rule = BlockStatementRule,
        count = 1,
        code = indoc! {r#"
            <?php

            if ($foo) {
                echo "Hello";
            } elseif ($bar)
                echo "World";
        "#}
    }

    test_lint_failure! {
        name = for_without_block,
        rule = BlockStatementRule,
        count = 1,
        code = indoc! {r#"
            <?php

            for ($i = 0; $i < 10; $i++)
                echo $i;
        "#}
    }

    test_lint_failure! {
        name = foreach_without_block,
        rule = BlockStatementRule,
        count = 1,
        code = indoc! {r#"
            <?php

            foreach ($items as $item)
                echo $item;
        "#}
    }

    test_lint_failure! {
        name = while_without_block,
        rule = BlockStatementRule,
        count = 1,
        code = indoc! {r#"
            <?php

            while ($foo)
                echo "Hello";
        "#}
    }

    test_lint_failure! {
        name = do_while_without_block,
        rule = BlockStatementRule,
        count = 1,
        code = indoc! {r#"
            <?php

            do
                echo "Hello";
            while ($foo);
        "#}
    }

    test_lint_failure! {
        name = else_without_block_not_else_if,
        rule = BlockStatementRule,
        count = 1,
        code = indoc! {r#"
            <?php

            if ($foo) {
                echo "Hello";
            } else
                echo "World";
        "#}
    }

    test_lint_success! {
        name = else_if_not_flagged,
        rule = BlockStatementRule,
        code = indoc! {r#"
            <?php

            if ($foo) {
                echo "Hello";
            } else if ($bar) {
                echo "World";
            }
        "#}
    }

    test_lint_success! {
        name = else_if_chain_not_flagged,
        rule = BlockStatementRule,
        code = indoc! {r#"
            <?php

            if ($a) {
                echo "a";
            } else if ($b) {
                echo "b";
            } else if ($c) {
                echo "c";
            }
        "#}
    }

    test_lint_failure! {
        name = else_if_without_block_in_inner_if,
        rule = BlockStatementRule,
        count = 1,
        code = indoc! {r#"
            <?php

            if ($foo) {
                echo "Hello";
            } else if ($bar)
                echo "World";  // <- this fails
        "#}
    }
}
