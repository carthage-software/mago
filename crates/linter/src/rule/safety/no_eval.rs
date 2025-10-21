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
pub struct NoEvalRule {
    meta: &'static RuleMeta,
    cfg: NoEvalConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoEvalConfig {
    pub level: Level,
}

impl Default for NoEvalConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoEvalConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoEvalRule {
    type Config = NoEvalConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Eval",
            code: "no-eval",
            description: indoc! {"
                Detects unsafe uses of the `eval` construct.
                The `eval` construct executes arbitrary code, which can be a major security risk if not used carefully.
            "},
            good_example: indoc! {r#"
                <?php

                // Safe alternative to eval
                $result = json_decode($jsonString);
            "#},
            bad_example: indoc! {r#"
                <?php

                eval('echo "Hello, world!";');
            "#},
            category: Category::Safety,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::EvalConstruct];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::EvalConstruct(eval_construct) = node else {
            return;
        };

        ctx.collector.report(
            Issue::new(self.cfg.level(), "Unsafe use of `eval` construct.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(eval_construct.eval.span)
                        .with_message("This `eval` construct is unsafe"),
                )
                .with_annotation(
                    Annotation::secondary(eval_construct.value.span())
                        .with_message("The evaluated code is here"),
                )
                .with_note("The `eval` construct executes arbitrary code, which can be a major security risk if not used carefully.")
                .with_note("It can potentially lead to remote code execution vulnerabilities if the evaluated code is not properly sanitized.")
                .with_note("Consider using safer alternatives whenever possible.")
                .with_help("Avoid using `eval` unless absolutely necessary, and ensure that any dynamically generated code is properly validated and sanitized before execution."),
        );
    }
}
