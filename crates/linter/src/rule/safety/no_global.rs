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

const GLOBALS_VARIABLE: &str = "$GLOBALS";

#[derive(Debug, Clone)]
pub struct NoGlobalRule {
    meta: &'static RuleMeta,
    cfg: NoGlobalConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoGlobalConfig {
    pub level: Level,
}

impl Default for NoGlobalConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoGlobalConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoGlobalRule {
    type Config = NoGlobalConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Global",
            code: "no-global",
            description: indoc! {"
                Detects the use of the `global` keyword and the `$GLOBALS` variable.

                The `global` keyword introduces global state into your function, making it harder to reason about and test.
            "},
            good_example: indoc! {r#"
                <?php

                function foo(string $bar): void {
                    // ...
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                function foo(): void {
                    global $bar;
                    // ...
                }
            "#},
            category: Category::Safety,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Global, NodeKind::DirectVariable];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        match node {
            Node::Global(global) => {
                let mut issue = Issue::new(self.cfg.level(), "Unsafe use of `global` keyword.")
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(global.global.span)
                        .with_message("This `global` keyword is used here"))
                    .with_note("The `global` keyword introduces global state into your function, making it harder to reason about and test.")
                    .with_note("It can also lead to unexpected behavior and make your code more prone to errors.")
                    .with_note("Consider using dependency injection or other techniques to manage state and avoid relying on global variables.")
                    .with_help("Refactor your code to avoid using the `global` keyword.");

                for variable in global.variables.iter() {
                    issue = issue.with_annotation(
                        Annotation::secondary(variable.span()).with_message("This variable is declared as global"),
                    );
                }

                ctx.collector.report(issue);
            }
            Node::DirectVariable(direct_variable) => {
                if !GLOBALS_VARIABLE.eq(direct_variable.name) {
                    return;
                }

                let issue = Issue::new(self.cfg.level(), "Unsafe use of `$GLOBALS` variable.")
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(direct_variable.span)
                        .with_message("The `$GLOBALS` variable is used here"))
                    .with_note("Accessing the `$GLOBALS` array directly can lead to similar issues as using the `global` keyword.")
                    .with_note("It can make your code harder to understand, test, and maintain due to the implicit global state.")
                    .with_note("Consider using dependency injection or other techniques to manage state and avoid relying on global variables.")
                    .with_help("Refactor your code to avoid using the `$GLOBALS` variable directly.");

                ctx.collector.report(issue);
            }
            _ => {}
        }
    }
}
