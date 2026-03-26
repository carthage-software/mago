use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
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
pub struct AmbiguousConstantAccessRule {
    meta: &'static RuleMeta,
    cfg: AmbiguousConstantAccessConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct AmbiguousConstantAccessConfig {
    pub level: Level,
}

impl Default for AmbiguousConstantAccessConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for AmbiguousConstantAccessConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for AmbiguousConstantAccessRule {
    type Config = AmbiguousConstantAccessConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Ambiguous Constant Access",
            code: "ambiguous-constant-access",
            description: indoc! {"
                Enforces that all constant references made from within a namespace are explicit.

                When an unqualified constant like `PHP_VERSION` is referenced from within a namespace,
                PHP performs a runtime fallback check (current namespace -> global namespace). This
                ambiguity can lead to unexpected behavior if a constant with the same name is later
                defined in the namespace.

                Making references explicit improves readability and prevents bugs.
            "},
            good_example: indoc! {r#"
                <?php

                namespace App;

                use const PHP_VERSION;

                // OK: Explicitly imported
                $version1 = PHP_VERSION;

                // OK: Explicitly global
                $version2 = \PHP_VERSION;
            "#},
            bad_example: indoc! {r#"
                <?php

                namespace App;

                // Ambiguous: could be App\PHP_VERSION or \PHP_VERSION
                $version = PHP_VERSION;
            "#},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::ConstantAccess];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        if ctx.scope.get_namespace().is_empty() {
            return;
        }

        let Node::ConstantAccess(access) = node else {
            return;
        };

        let identifier = access.name;

        if identifier.is_qualified() || identifier.is_fully_qualified() {
            return;
        }

        if ctx.is_name_imported(&identifier) {
            return;
        }

        let constant_name = identifier.value();

        // Skip compile-time constants that PHP resolves without ambiguity.
        let name_lower = constant_name.to_ascii_lowercase();
        if matches!(name_lower.as_str(), "true" | "false" | "null" | "stdin" | "stdout" | "stderr") {
            return;
        }

        ctx.collector.report(
            Issue::new(self.cfg.level, "Ambiguous constant access detected.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(identifier.span())
                        .with_message(format!("The reference to `{constant_name}` is ambiguous")),
                )
                .with_note("At runtime, PHP cannot be sure if this refers to a namespaced constant or the global one.")
                .with_note("Making references explicit improves code clarity and prevents bugs if a constant with the same name is later added to the namespace.")
                .with_help(format!("Make the reference explicit: use `\\{constant_name}` or add a `use const {constant_name};` statement.")),
        );
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::AmbiguousConstantAccessRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = imported_constant_is_not_ambiguous,
        rule = AmbiguousConstantAccessRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use const PHP_VERSION;

            $version = PHP_VERSION;
        "#}
    }

    test_lint_success! {
        name = fully_qualified_constant_is_not_ambiguous,
        rule = AmbiguousConstantAccessRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $version = \PHP_VERSION;
        "#}
    }

    test_lint_success! {
        name = global_scope_constant_is_not_ambiguous,
        rule = AmbiguousConstantAccessRule,
        code = indoc! {r#"
            <?php

            $version = PHP_VERSION;
        "#}
    }

    test_lint_success! {
        name = compile_time_constants_are_not_flagged,
        rule = AmbiguousConstantAccessRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $a = true;
            $b = false;
            $c = null;
            $d = STDIN;
            $e = STDOUT;
            $f = STDERR;
        "#}
    }

    test_lint_failure! {
        name = unqualified_constant_in_namespace,
        rule = AmbiguousConstantAccessRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $version = PHP_VERSION;
        "#}
    }

    test_lint_failure! {
        name = multiple_unqualified_constants,
        rule = AmbiguousConstantAccessRule,
        count = 2,
        code = indoc! {r#"
            <?php

            namespace App;

            $a = PHP_VERSION;
            $b = PHP_INT_MAX;
        "#}
    }
}
