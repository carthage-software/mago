use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoGotoRule {
    meta: &'static RuleMeta,
    cfg: NoGotoConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoGotoConfig {
    pub level: Level,
}

impl Default for NoGotoConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for NoGotoConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoGotoRule {
    type Config = NoGotoConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Goto",
            code: "no-goto",
            description: indoc! {r#"
                Detects the use of `goto` statements and labels. The `goto` statement can make
                code harder to read, understand, and maintain. It can lead to "spaghetti code"
                and make it difficult to follow the flow of execution.
            "#},
            good_example: indoc! {r#"
                <?php

                $i = 0;
                while ($i < 10) {
                    if ($i === 5) {
                        break; // Structured control flow.
                    }
                    $i++;
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                $i = 0;
                loop:
                if ($i >= 10) {
                    goto end;
                }

                $i++;
                goto loop;
                end:
            "#},
            category: Category::Maintainability,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Goto, NodeKind::Label];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        match node {
            Node::Goto(goto) => {
                let issue = Issue::new(self.cfg.level, "Avoid using `goto`.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(goto.goto.span())
                            .with_message("This `goto` statement creates a non-linear control flow"),
                    )
                    .with_annotation(Annotation::secondary(goto.label.span()).with_message("Jumping to this label"))
                    .with_note(
                        "The `goto` statement makes code harder to read and maintain by breaking structured control flow.",
                    )
                    .with_help("Refactor the code to use loops or conditional statements instead of `goto`.");

                ctx.collector.report(issue);
            }
            Node::Label(label) => {
                let issue = Issue::new(self.cfg.level, "`goto` labels are discouraged.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(label.span())
                            .with_message(format!("Label `{}` is declared here", label.name.value)),
                    )
                    .with_note(
                        "Labels are used with `goto` statements, which can lead to confusing 'spaghetti code'.",
                    )
                    .with_help("Refactor the code to use structured control flow like loops or functions to eliminate the need for this label.");

                ctx.collector.report(issue);
            }
            _ => {}
        }
    }
}
