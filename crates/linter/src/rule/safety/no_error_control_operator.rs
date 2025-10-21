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
pub struct NoErrorControlOperatorRule {
    meta: &'static RuleMeta,
    cfg: NoErrorControlOperatorConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoErrorControlOperatorConfig {
    pub level: Level,
}

impl Default for NoErrorControlOperatorConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoErrorControlOperatorConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoErrorControlOperatorRule {
    type Config = NoErrorControlOperatorConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Error Control Operator",
            code: "no-error-control-operator",
            description: indoc! {"
                Detects the use of the error control operator `@`.

                The error control operator suppresses errors and makes debugging more difficult.
            "},
            good_example: indoc! {r#"
                <?php

                try {
                    $result = file_get_contents('example.txt');
                } catch (Throwable $e) {
                    // Handle error
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                $result = @file_get_contents('example.txt');
            "#},
            category: Category::Safety,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::UnaryPrefix];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::UnaryPrefix(unary_prefix) = node else {
            return;
        };

        if let UnaryPrefixOperator::ErrorControl(_) = unary_prefix.operator {
            let issue = Issue::new(self.cfg.level(), "Unsafe use of error control operator `@`.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(unary_prefix.operator.span()).with_message("This operator suppresses errors"),
                )
                .with_annotation(
                    Annotation::secondary(unary_prefix.operand.span())
                        .with_message("This expression is being suppressed"),
                )
                .with_note("Error control operator hide potential errors and make debugging more difficult.")
                .with_help("Remove the `@` and use `set_error_handler` to handle errors instead.");

            ctx.collector.report(issue);
        }
    }
}
