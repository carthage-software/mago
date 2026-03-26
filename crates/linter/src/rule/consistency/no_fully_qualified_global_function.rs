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
pub struct NoFullyQualifiedGlobalFunctionRule {
    meta: &'static RuleMeta,
    cfg: NoFullyQualifiedGlobalFunctionConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoFullyQualifiedGlobalFunctionConfig {
    pub level: Level,
}

impl Default for NoFullyQualifiedGlobalFunctionConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoFullyQualifiedGlobalFunctionConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoFullyQualifiedGlobalFunctionRule {
    type Config = NoFullyQualifiedGlobalFunctionConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Fully Qualified Global Function",
            code: "no-fully-qualified-global-function",
            description: indoc! {"
                Disallows fully-qualified references to global functions within a namespace.

                Instead of using the backslash prefix (e.g., `\\strlen()`),
                prefer an explicit `use function` import statement. This improves
                readability and keeps imports centralized at the top of the file.
            "},
            good_example: indoc! {r#"
                <?php

                namespace App;

                use function strlen;

                $length = strlen("hello");
            "#},
            bad_example: indoc! {r#"
                <?php

                namespace App;

                $length = \strlen("hello");
            "#},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        if ctx.scope.get_namespace().is_empty() {
            return;
        }

        let Node::FunctionCall(call) = node else {
            return;
        };

        let Expression::Identifier(identifier) = call.function else {
            return;
        };

        if !identifier.is_fully_qualified() {
            return;
        }

        let function_name = identifier.value().trim_start_matches('\\');

        ctx.collector.report(
            Issue::new(self.cfg.level, "Fully-qualified function call detected.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(identifier.span())
                        .with_message(format!("The call to `\\{function_name}` uses a fully-qualified name")),
                )
                .with_note("Fully-qualified function calls bypass the import system, making it harder to see which global functions a file depends on.")
                .with_help(format!("Add `use function {function_name};` and call `{function_name}(...)` directly.")),
        );
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoFullyQualifiedGlobalFunctionRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = imported_function_is_not_flagged,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use function strlen;

            $length = strlen("hello");
        "#}
    }

    test_lint_success! {
        name = unqualified_function_is_not_flagged,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $length = strlen("hello");
        "#}
    }

    test_lint_success! {
        name = global_scope_fq_function_is_not_flagged,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            $length = \strlen("hello");
        "#}
    }

    test_lint_failure! {
        name = fq_function_call_in_namespace,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $length = \strlen("hello");
        "#}
    }
}
