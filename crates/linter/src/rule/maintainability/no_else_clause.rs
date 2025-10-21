use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoElseClauseRule {
    meta: &'static RuleMeta,
    cfg: NoElseClauseConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoElseClauseConfig {
    pub level: Level,
}

impl Default for NoElseClauseConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoElseClauseConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoElseClauseRule {
    type Config = NoElseClauseConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Else Clause",
            code: "no-else-clause",
            description: indoc! {r#"
                Flags `if` statements that include an `else` or `elseif` branch.

                Using `else` or `elseif` can lead to deeply nested code and complex control flow.
                This can often be simplified by using early returns (guard clauses), which makes
                the code easier to read and maintain by reducing its cyclomatic complexity.
            "#},
            good_example: indoc! {r#"
                <?php

                function process($user) {
                    if (!$user->isVerified()) {
                        return; // Early return
                    }

                    // "Happy path" continues here
                    $user->login();
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                function process($user) {
                    if ($user->isVerified()) {
                        // "Happy path" is nested
                        $user->login();
                    } else {
                        // Logic is split across branches
                        return;
                    }
                }
            "#},
            category: Category::Maintainability,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::If];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::If(if_stmt) = node else {
            return;
        };

        let mut report_on_clauses = |else_if_clauses: &[Span], else_clause: Option<Span>| {
            for clause in else_if_clauses {
                let issue = Issue::new(self.cfg.level, "Avoid `elseif` clauses.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(clause.span())
                            .with_message("This `elseif` adds unnecessary complexity"),
                    )
                    .with_note("Complex conditional chains can often be simplified by using early returns or a `match` expression.")
                    .with_help("Refactor to use guard clauses (early returns) or a `match` expression for clarity.");

                ctx.collector.report(issue);
            }

            if let Some(clause) = else_clause {
                let issue = Issue::new(self.cfg.level, "Avoid `else` clauses.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(clause.span()).with_message("This `else` clause can often be eliminated"),
                    )
                    .with_note("Code is often clearer when the main logic is not nested inside an `if` statement.")
                    .with_help(
                        "Consider refactoring to use an early return (a guard clause) to simplify the control flow.",
                    );

                ctx.collector.report(issue);
            }
        };

        match &if_stmt.body {
            IfBody::Statement(body) => {
                report_on_clauses(
                    body.else_if_clauses.iter().map(|e| e.span()).collect::<Vec<_>>().as_slice(),
                    body.else_clause.as_ref().map(|e| e.span()),
                );
            }
            IfBody::ColonDelimited(body) => {
                report_on_clauses(
                    body.else_if_clauses.iter().map(|e| e.span()).collect::<Vec<_>>().as_slice(),
                    body.else_clause.as_ref().map(|e| e.span()),
                );
            }
        }
    }
}
