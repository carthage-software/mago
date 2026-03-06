use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Continue;
use mago_syntax::ast::DeclareBody;
use mago_syntax::ast::Expression;
use mago_syntax::ast::IfBody;
use mago_syntax::ast::Literal;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Statement;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct SwitchContinueToBreakRule {
    meta: &'static RuleMeta,
    cfg: SwitchContinueToBreakConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct SwitchContinueToBreakConfig {
    pub level: Level,
}

impl Default for SwitchContinueToBreakConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for SwitchContinueToBreakConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for SwitchContinueToBreakRule {
    type Config = SwitchContinueToBreakConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Switch Continue to Break",
            code: "switch-continue-to-break",
            description: indoc! {"
                Detects the use of `continue` inside a `switch` statement, which should
                be `break` instead.

                In PHP, `continue` inside a `switch` behaves the same as `break`, but
                using `continue` is misleading because it suggests continuing a loop.
            "},
            good_example: indoc! {r"
                <?php

                switch ($value) {
                    case 1:
                        echo 'one';
                        break;
                }
            "},
            bad_example: indoc! {r"
                <?php

                switch ($value) {
                    case 1:
                        echo 'one';
                        continue;
                }
            "},
            category: Category::Correctness,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Switch];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Switch(switch) = node else {
            return;
        };

        for case in switch.body.cases() {
            for r#continue in find_switch_continues_in_statements(case.statements()) {
                let issue =
                    Issue::new(self.cfg.level(), "Use `break` instead of `continue` inside a `switch` statement")
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(r#continue.r#continue.span())
                                .with_message("`continue` behaves the same as `break` in a switch"),
                        )
                        .with_note("In PHP, `continue` inside a `switch` is equivalent to `break`, but is misleading.")
                        .with_help("Replace `continue` with `break`");

                ctx.collector.propose(issue, |edits| {
                    edits.push(TextEdit::replace(r#continue.r#continue.span(), "break"));
                });
            }
        }
    }
}

#[inline]
fn find_switch_continues_in_statements<'ast, 'arena>(
    statements: &'ast [Statement<'arena>],
) -> Vec<&'ast Continue<'arena>> {
    let mut continues = vec![];

    for statement in statements {
        continues.extend(find_switch_continues_in_statement(statement));
    }

    continues
}

#[inline]
fn find_switch_continues_in_statement<'ast, 'arena>(statement: &'ast Statement<'arena>) -> Vec<&'ast Continue<'arena>> {
    match statement {
        Statement::Namespace(namespace) => find_switch_continues_in_statements(namespace.statements().as_slice()),
        Statement::Block(block) => find_switch_continues_in_statements(block.statements.as_slice()),
        Statement::Declare(declare) => match &declare.body {
            DeclareBody::Statement(statement) => find_switch_continues_in_statement(statement),
            DeclareBody::ColonDelimited(body) => find_switch_continues_in_statements(body.statements.as_slice()),
        },
        Statement::Try(r#try) => {
            let mut continues = find_switch_continues_in_statements(r#try.block.statements.as_slice());

            for catch_clause in &r#try.catch_clauses {
                continues.extend(find_switch_continues_in_statements(catch_clause.block.statements.as_slice()));
            }

            if let Some(finally_clause) = &r#try.finally_clause {
                continues.extend(find_switch_continues_in_statements(finally_clause.block.statements.as_slice()));
            }

            continues
        }
        Statement::If(r#if) => match &r#if.body {
            IfBody::Statement(body) => {
                let mut continues = find_switch_continues_in_statement(body.statement);

                for else_if_clause in &body.else_if_clauses {
                    continues.extend(find_switch_continues_in_statement(else_if_clause.statement));
                }

                if let Some(else_clause) = &body.else_clause {
                    continues.extend(find_switch_continues_in_statement(else_clause.statement));
                }

                continues
            }
            IfBody::ColonDelimited(body) => {
                let mut continues = find_switch_continues_in_statements(body.statements.as_slice());

                for else_if_clause in &body.else_if_clauses {
                    continues.extend(find_switch_continues_in_statements(else_if_clause.statements.as_slice()));
                }

                if let Some(else_clause) = &body.else_clause {
                    continues.extend(find_switch_continues_in_statements(else_clause.statements.as_slice()));
                }

                continues
            }
        },
        Statement::Continue(r#continue) if continue_targets_switch(r#continue) => vec![r#continue],
        Statement::Foreach(_)
        | Statement::For(_)
        | Statement::While(_)
        | Statement::DoWhile(_)
        | Statement::Switch(_)
        | Statement::Function(_)
        | Statement::Class(_)
        | Statement::Interface(_)
        | Statement::Trait(_)
        | Statement::Enum(_) => vec![],
        _ => vec![],
    }
}

#[inline]
fn continue_targets_switch(r#continue: &Continue<'_>) -> bool {
    match r#continue.level {
        None => true,
        Some(Expression::Literal(Literal::Integer(literal))) => literal.value == Some(1),
        Some(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::SwitchContinueToBreakRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = break_in_switch_is_ok,
        rule = SwitchContinueToBreakRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    echo 'one';
                    break;
                default:
                    echo 'other';
                    break;
            }
        "}
    }

    test_lint_failure! {
        name = continue_in_switch_is_bad,
        rule = SwitchContinueToBreakRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    echo 'one';
                    continue;
            }
        "}
    }

    test_lint_failure! {
        name = continue_1_in_switch_is_bad,
        rule = SwitchContinueToBreakRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    echo 'one';
                    continue 1;
            }
        "}
    }

    test_lint_success! {
        name = continue_2_in_switch_is_ok,
        rule = SwitchContinueToBreakRule,
        code = indoc! {r"
            <?php

            foreach ($items as $item) {
                switch ($item) {
                    case 1:
                        continue 2;
                }
            }
        "}
    }

    test_lint_failure! {
        name = nested_continue_in_switch_is_bad,
        rule = SwitchContinueToBreakRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    if ($cond) {
                        continue;
                    }
                    break;
            }
        "}
    }

    test_lint_success! {
        name = continue_inside_nested_loop_is_ok,
        rule = SwitchContinueToBreakRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    while ($cond) {
                        continue;
                    }
                    break;
            }
        "}
    }
}
