use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::IfBody;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::utils::condition::is_falsy;
use mago_syntax::utils::condition::is_truthy;
use mago_syntax::utils::definition::statement_contains_only_definitions;
use mago_syntax::utils::definition::statement_sequence_contains_only_definitions;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct ConstantConditionRule {
    meta: &'static RuleMeta,
    cfg: ConstantConditionConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ConstantConditionConfig {
    pub level: Level,
}

impl Default for ConstantConditionConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for ConstantConditionConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ConstantConditionRule {
    type Config = ConstantConditionConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Constant Condition",
            code: "constant-condition",
            description: indoc! {"
                Detects `if` statements where the condition is a constant that always
                evaluates to `true` or `false`.

                Such statements are redundant. If the condition is always `true`, the `if`
                wrapper is unnecessary. If it's always `false`, the enclosed code is dead
                and can be removed or refactored.
            "},
            good_example: indoc! {r#"
                <?php
                if ($variable > 10) {
                    echo "Greater than 10";
                }
            "#},
            bad_example: indoc! {r#"
                <?php
                if (true) {
                    echo "This will always run";
                }

                if (false) {
                    echo "This is dead code";
                }
            "#},
            category: Category::Redundancy,

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

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::If(r#if) = node else {
            return;
        };

        if is_truthy(r#if.condition) {
            let issue = Issue::new(self.cfg.level, "Redundant `if` statement.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(r#if.condition.span()).with_message("This condition is always `true`"),
                )
                .with_note("The `if` wrapper is unnecessary as this code block will always be executed.")
                .with_help("Remove the `if` statement and unwrap the code block.");

            ctx.collector.propose(issue, |edits| {
                edits.push(TextEdit::delete(r#if.r#if.span.join(r#if.right_parenthesis)));

                match &r#if.body {
                    IfBody::Statement(body) => {
                        for clause in &body.else_if_clauses {
                            edits.push(TextEdit::delete(clause.span()));
                        }

                        if let Some(else_clause) = &body.else_clause {
                            edits.push(TextEdit::delete(else_clause.span()));
                        }
                    }
                    IfBody::ColonDelimited(body) => {
                        edits.push(TextEdit::delete(body.colon));

                        for clause in &body.else_if_clauses {
                            edits.push(TextEdit::delete(clause.span()));
                        }

                        if let Some(else_clause) = &body.else_clause {
                            edits.push(TextEdit::delete(else_clause.span()));
                        }

                        edits.push(TextEdit::delete(body.endif.span()));
                        edits.push(TextEdit::delete(body.terminator.span()));
                    }
                }
            });

            return;
        }

        if is_falsy(r#if.condition) {
            // Exclude `if (false)` blocks used for IDE stubs.
            match &r#if.body {
                IfBody::Statement(body)
                    if body.else_if_clauses.is_empty()
                        && body.else_clause.is_none()
                        && statement_contains_only_definitions(body.statement) =>
                {
                    return;
                }
                IfBody::ColonDelimited(body)
                    if body.else_if_clauses.is_empty()
                        && body.else_clause.is_none()
                        && statement_sequence_contains_only_definitions(&body.statements) =>
                {
                    return;
                }
                _ => {}
            }

            let issue = Issue::new(self.cfg.level, "Redundant `if` statement.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(r#if.condition.span()).with_message("This condition is always `false`"),
                )
                .with_note("The body of this `if` is dead code as the condition is never met.")
                .with_help("Remove the `if` statement. The `else` or `elseif` branches may need to be promoted.");

            ctx.collector.propose(issue, |edits| match &r#if.body {
                IfBody::Statement(body) => {
                    if let Some(else_if_clause) = body.else_if_clauses.first() {
                        let span = r#if.r#if.span.join(else_if_clause.elseif.span());

                        edits.push(TextEdit::delete(span.start_offset()..(span.end_offset() - 2)));
                    } else if let Some(else_clause) = &body.else_clause {
                        let span = r#if.r#if.span.join(else_clause.r#else.span());

                        edits.push(TextEdit::delete(span));
                    } else {
                        edits.push(TextEdit::delete(r#if.span()));
                    }
                }
                IfBody::ColonDelimited(body) => {
                    if let Some(else_if_clause) = body.else_if_clauses.first() {
                        let span = r#if.r#if.span.join(else_if_clause.elseif.span());

                        edits.push(TextEdit::delete(span.start_offset()..(span.end_offset() - 2)));
                    } else if let Some(else_clause) = &body.else_clause {
                        edits.push(TextEdit::delete(r#if.r#if.span.join(else_clause.colon.span())));
                        edits.push(TextEdit::delete(body.endif.span()));
                        edits.push(TextEdit::delete(body.terminator.span()));
                    } else {
                        edits.push(TextEdit::delete(r#if.span()));
                    }
                }
            });
        }
    }
}
