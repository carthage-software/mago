use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoDirectExceptionThrowRule {
    meta: &'static RuleMeta,
    cfg: NoDirectExceptionThrowConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoDirectExceptionThrowConfig {
    pub level: Level,
}

impl Default for NoDirectExceptionThrowConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoDirectExceptionThrowConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoDirectExceptionThrowRule {
    type Config = NoDirectExceptionThrowConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Direct Exception Throw",
            code: "no-direct-exception-throw",
            description: indoc! {"
                Flags direct throwing of the generic `\\Exception` base class. Use context-specific
                exception types instead (e.g. `InvalidArgumentException`, `RuntimeException`,
                or custom exception classes) for better error handling and debugging.
            "},
            good_example: indoc! {r"
                <?php

                throw new \InvalidArgumentException('Invalid value');
            "},
            bad_example: indoc! {r"
                <?php

                throw new \Exception('Something went wrong');
            "},
            category: Category::BestPractices,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Throw];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Throw(throw) = node else {
            return;
        };

        // Check if the exception is `new Exception(...)`
        let Expression::Instantiation(instantiation) = throw.exception else {
            return;
        };

        let Expression::Identifier(class_id) = instantiation.class else {
            return;
        };

        let fqcn = ctx.lookup_name(class_id);

        // Only flag the base \Exception class, not subclasses
        if !fqcn.eq_ignore_ascii_case("Exception") {
            return;
        }

        ctx.collector.report(
            Issue::new(
                self.cfg.level(),
                "Throwing the generic `\\Exception` class directly is discouraged.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(throw.span())
                    .with_message("Use a more specific exception type"),
            )
            .with_help(
                "Use a context-specific exception type like `\\InvalidArgumentException`, \
                 `\\RuntimeException`, or a custom exception class.",
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = specific_exception,
        rule = NoDirectExceptionThrowRule,
        code = r#"
            <?php

            throw new \InvalidArgumentException('Invalid value');
        "#
    }

    test_lint_success! {
        name = custom_exception,
        rule = NoDirectExceptionThrowRule,
        code = r#"
            <?php

            throw new \App\Exception\CustomException('Error');
        "#
    }

    test_lint_failure! {
        name = generic_exception_fqn,
        rule = NoDirectExceptionThrowRule,
        code = r#"
            <?php

            throw new \Exception('Something went wrong');
        "#
    }

    test_lint_failure! {
        name = generic_exception_bare,
        rule = NoDirectExceptionThrowRule,
        code = r#"
            <?php

            throw new Exception('Something went wrong');
        "#
    }
}
