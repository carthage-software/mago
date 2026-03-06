use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Construct;
use mago_syntax::ast::DeclareBody;
use mago_syntax::ast::Expression;
use mago_syntax::ast::IfBody;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Statement;
use mago_syntax::ast::SwitchCase;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoBreakCommentRule {
    meta: &'static RuleMeta,
    cfg: NoBreakCommentConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoBreakCommentConfig {
    pub level: Level,
}

impl Default for NoBreakCommentConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoBreakCommentConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoBreakCommentRule {
    type Config = NoBreakCommentConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Break Comment",
            code: "no-break-comment",
            description: indoc! {"
                Requires a `// no break` comment when a `switch` case falls through
                to the next case without a `break`, `return`, `continue`, or `throw`.

                Fall-through cases without a comment are ambiguous — readers cannot tell
                if the fall-through is intentional or a missing `break` bug.
            "},
            good_example: indoc! {r"
                <?php

                switch ($value) {
                    case 1:
                        echo 'one';
                        // no break
                    case 2:
                        echo 'two';
                        break;
                }
            "},
            bad_example: indoc! {r"
                <?php

                switch ($value) {
                    case 1:
                        echo 'one';
                    case 2:
                        echo 'two';
                        break;
                }
            "},
            category: Category::Safety,
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

        let cases_vec: Vec<_> = switch.body.cases().iter().collect();

        // Check all cases except the last one (last case cannot fall through)
        for (i, case) in cases_vec.iter().enumerate() {
            if i == cases_vec.len() - 1 {
                break;
            }

            let statements = case.statements();

            // Empty case body is allowed (grouping cases)
            if statements.is_empty() {
                continue;
            }

            if statements_terminate_switch_case(statements) {
                continue;
            }

            // Case falls through — check for "no break" comment in source.
            // We look from the current case start to the next case start, because
            // comments like `// no break` sit between the last statement and the next case keyword,
            // outside the current case's AST span.
            let source_code = ctx.source_file.contents.as_ref();
            let region_start = case.span().start_offset() as usize;
            let region_end = cases_vec[i + 1].span().start_offset() as usize;
            let case_text = &source_code[region_start..region_end];

            let case_lower = case_text.to_ascii_lowercase();
            let has_no_break_comment = case_lower.contains("// no break")
                || case_lower.contains("// no-break")
                || case_lower.contains("/* no break")
                || case_lower.contains("/* no-break")
                || case_lower.contains("// fall through")
                || case_lower.contains("// fall-through")
                || case_lower.contains("// fallthrough")
                || case_lower.contains("/* fall through")
                || case_lower.contains("/* fall-through")
                || case_lower.contains("/* fallthrough")
                || case_lower.contains("# no break")
                || case_lower.contains("# no-break")
                || case_lower.contains("# fall through")
                || case_lower.contains("# fall-through")
                || case_lower.contains("# fallthrough");

            if !has_no_break_comment {
                let case_keyword_span = match case {
                    SwitchCase::Expression(expr_case) => expr_case.case.span(),
                    SwitchCase::Default(default_case) => default_case.default.span(),
                };

                let issue = Issue::new(self.cfg.level(), "Switch case falls through without a `// no break` comment")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(case_keyword_span).with_message("This case falls through to the next case"),
                    )
                    .with_note("Intentional fall-through should be documented with a `// no break` comment.")
                    .with_help("Add a `// no break` comment at the end of the case, or add a `break` statement");

                ctx.collector.report(issue);
            }
        }
    }
}

#[inline]
fn statements_terminate_switch_case(statements: &[Statement<'_>]) -> bool {
    for statement in statements {
        if statement_terminates_switch_case(statement) {
            return true;
        }
    }

    false
}

#[inline]
fn statement_terminates_switch_case(statement: &Statement<'_>) -> bool {
    match statement {
        Statement::Namespace(namespace) => statements_terminate_switch_case(namespace.statements().as_slice()),
        Statement::Block(block) => statements_terminate_switch_case(block.statements.as_slice()),
        Statement::Declare(declare) => match &declare.body {
            DeclareBody::Statement(statement) => statement_terminates_switch_case(statement),
            DeclareBody::ColonDelimited(body) => statements_terminate_switch_case(body.statements.as_slice()),
        },
        Statement::Try(r#try) => {
            if r#try.finally_clause.as_ref().is_some_and(|finally_clause| {
                statements_terminate_switch_case(finally_clause.block.statements.as_slice())
            }) {
                return true;
            }

            statements_terminate_switch_case(r#try.block.statements.as_slice())
                && r#try
                    .catch_clauses
                    .iter()
                    .all(|catch_clause| statements_terminate_switch_case(catch_clause.block.statements.as_slice()))
        }
        Statement::If(r#if) => if_statement_terminates_switch_case(&r#if.body),
        Statement::Break(_) | Statement::Continue(_) | Statement::Return(_) | Statement::Goto(_) => true,
        Statement::Expression(expression_statement) => {
            expression_terminates_switch_case(expression_statement.expression)
        }
        Statement::Foreach(_)
        | Statement::For(_)
        | Statement::While(_)
        | Statement::DoWhile(_)
        | Statement::Switch(_)
        | Statement::Function(_)
        | Statement::Class(_)
        | Statement::Interface(_)
        | Statement::Trait(_)
        | Statement::Enum(_) => false,
        _ => false,
    }
}

#[inline]
fn if_statement_terminates_switch_case(body: &IfBody<'_>) -> bool {
    match body {
        IfBody::Statement(body) => {
            statement_terminates_switch_case(body.statement)
                && body
                    .else_if_clauses
                    .iter()
                    .all(|else_if_clause| statement_terminates_switch_case(else_if_clause.statement))
                && body
                    .else_clause
                    .as_ref()
                    .is_some_and(|else_clause| statement_terminates_switch_case(else_clause.statement))
        }
        IfBody::ColonDelimited(body) => {
            statements_terminate_switch_case(body.statements.as_slice())
                && body
                    .else_if_clauses
                    .iter()
                    .all(|else_if_clause| statements_terminate_switch_case(else_if_clause.statements.as_slice()))
                && body
                    .else_clause
                    .as_ref()
                    .is_some_and(|else_clause| statements_terminate_switch_case(else_clause.statements.as_slice()))
        }
    }
}

#[inline]
fn expression_terminates_switch_case(expression: &Expression<'_>) -> bool {
    matches!(
        expression,
        Expression::Throw(_) | Expression::Construct(Construct::Exit(_)) | Expression::Construct(Construct::Die(_))
    )
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoBreakCommentRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = break_in_all_cases_is_ok,
        rule = NoBreakCommentRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    echo 'one';
                    break;
                case 2:
                    echo 'two';
                    break;
            }
        "}
    }

    test_lint_success! {
        name = no_break_comment_is_ok,
        rule = NoBreakCommentRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    echo 'one';
                    // no break
                case 2:
                    echo 'two';
                    break;
            }
        "}
    }

    test_lint_success! {
        name = empty_case_is_ok,
        rule = NoBreakCommentRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                case 2:
                    echo 'one or two';
                    break;
            }
        "}
    }

    test_lint_failure! {
        name = fall_through_without_comment_is_bad,
        rule = NoBreakCommentRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    echo 'one';
                case 2:
                    echo 'two';
                    break;
            }
        "}
    }

    test_lint_success! {
        name = return_in_case_is_ok,
        rule = NoBreakCommentRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    return 'one';
                case 2:
                    return 'two';
            }
        "}
    }

    test_lint_success! {
        name = exit_in_case_is_ok,
        rule = NoBreakCommentRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    exit(1);
                case 2:
                    echo 'two';
                    break;
            }
        "}
    }

    test_lint_success! {
        name = die_in_case_is_ok,
        rule = NoBreakCommentRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    die('fatal');
                case 2:
                    echo 'two';
                    break;
            }
        "}
    }

    test_lint_success! {
        name = hash_no_break_comment_is_ok,
        rule = NoBreakCommentRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    echo 'one';
                    # no break
                case 2:
                    echo 'two';
                    break;
            }
        "}
    }

    test_lint_success! {
        name = block_comment_fall_through_is_ok,
        rule = NoBreakCommentRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    echo 'one';
                    /* fall through */
                case 2:
                    echo 'two';
                    break;
            }
        "}
    }

    test_lint_success! {
        name = throw_in_case_is_ok,
        rule = NoBreakCommentRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    throw new \Exception('error');
                case 2:
                    echo 'two';
                    break;
            }
        "}
    }

    test_lint_success! {
        name = continue_in_case_is_ok,
        rule = NoBreakCommentRule,
        code = indoc! {r"
            <?php

            foreach ($items as $item) {
                switch ($item) {
                    case 1:
                        continue 2;
                    case 2:
                        echo 'two';
                        break;
                }
            }
        "}
    }

    test_lint_failure! {
        name = default_fallthrough_without_comment_is_bad,
        rule = NoBreakCommentRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                default:
                    echo 'default';
                case 1:
                    echo 'one';
                    break;
            }
        "}
    }

    test_lint_success! {
        name = goto_in_case_is_ok,
        rule = NoBreakCommentRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    goto done;
                case 2:
                    echo 'two';
                    break;
            }
            done:
            echo 'done';
        "}
    }

    test_lint_success! {
        name = nested_if_terminator_is_ok,
        rule = NoBreakCommentRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    if ($cond) {
                        break;
                    } else {
                        return 'one';
                    }
                case 2:
                    return 'two';
            }
        "}
    }

    test_lint_failure! {
        name = conditional_break_without_else_is_bad,
        rule = NoBreakCommentRule,
        code = indoc! {r"
            <?php

            switch ($value) {
                case 1:
                    if ($cond) {
                        break;
                    }
                case 2:
                    return 'two';
            }
        "}
    }
}
