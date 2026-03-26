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
pub struct NoFullyQualifiedGlobalRule {
    meta: &'static RuleMeta,
    cfg: NoFullyQualifiedGlobalConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoFullyQualifiedGlobalConfig {
    pub level: Level,
}

impl Default for NoFullyQualifiedGlobalConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoFullyQualifiedGlobalConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoFullyQualifiedGlobalRule {
    type Config = NoFullyQualifiedGlobalConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Fully Qualified Global",
            code: "no-fully-qualified-global",
            description: indoc! {"
                Disallows fully-qualified references to global functions and constants within a namespace.

                Instead of using the backslash prefix (e.g., `\\strlen()` or `\\PHP_VERSION`),
                prefer explicit `use function` or `use const` import statements. This improves
                readability and keeps imports centralized at the top of the file.
            "},
            good_example: indoc! {r#"
                <?php

                namespace App;

                use function strlen;
                use const PHP_VERSION;

                $length = strlen("hello");
                $version = PHP_VERSION;
            "#},
            bad_example: indoc! {r#"
                <?php

                namespace App;

                $length = \strlen("hello");
                $version = \PHP_VERSION;
            "#},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionCall, NodeKind::ConstantAccess];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        if ctx.scope.get_namespace().is_empty() {
            return;
        }

        match node {
            Node::FunctionCall(call) => {
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
            Node::ConstantAccess(access) => {
                let identifier = access.name;

                if !identifier.is_fully_qualified() {
                    return;
                }

                let constant_name = identifier.value().trim_start_matches('\\');

                // Skip true, false, null — these are language keywords.
                let name_lower = constant_name.to_ascii_lowercase();
                if matches!(name_lower.as_str(), "true" | "false" | "null") {
                    return;
                }

                ctx.collector.report(
                    Issue::new(self.cfg.level, "Fully-qualified constant access detected.")
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(identifier.span())
                                .with_message(format!("The constant `\\{constant_name}` uses a fully-qualified name")),
                        )
                        .with_note("Fully-qualified constant access bypasses the import system, making it harder to see which global constants a file depends on.")
                        .with_help(format!("Add `use const {constant_name};` and reference `{constant_name}` directly.")),
                );
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoFullyQualifiedGlobalRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = imported_function_is_not_flagged,
        rule = NoFullyQualifiedGlobalRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use function strlen;

            $length = strlen("hello");
        "#}
    }

    test_lint_success! {
        name = imported_constant_is_not_flagged,
        rule = NoFullyQualifiedGlobalRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use const PHP_VERSION;

            $version = PHP_VERSION;
        "#}
    }

    test_lint_success! {
        name = unqualified_function_is_not_flagged,
        rule = NoFullyQualifiedGlobalRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $length = strlen("hello");
        "#}
    }

    test_lint_success! {
        name = global_scope_fq_is_not_flagged,
        rule = NoFullyQualifiedGlobalRule,
        code = indoc! {r#"
            <?php

            $length = \strlen("hello");
            $version = \PHP_VERSION;
        "#}
    }

    test_lint_success! {
        name = fq_true_false_null_not_flagged,
        rule = NoFullyQualifiedGlobalRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $a = \true;
            $b = \false;
            $c = \null;
        "#}
    }

    test_lint_failure! {
        name = fq_function_call_in_namespace,
        rule = NoFullyQualifiedGlobalRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $length = \strlen("hello");
        "#}
    }

    test_lint_failure! {
        name = fq_constant_access_in_namespace,
        rule = NoFullyQualifiedGlobalRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $version = \PHP_VERSION;
        "#}
    }

    test_lint_failure! {
        name = multiple_fq_references,
        rule = NoFullyQualifiedGlobalRule,
        count = 2,
        code = indoc! {r#"
            <?php

            namespace App;

            $length = \strlen("hello");
            $version = \PHP_VERSION;
        "#}
    }
}
