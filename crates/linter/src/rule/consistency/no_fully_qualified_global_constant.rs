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
use mago_text_edit::Safety;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoFullyQualifiedGlobalConstantRule {
    meta: &'static RuleMeta,
    cfg: NoFullyQualifiedGlobalConstantConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoFullyQualifiedGlobalConstantConfig {
    pub level: Level,
}

impl Default for NoFullyQualifiedGlobalConstantConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoFullyQualifiedGlobalConstantConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoFullyQualifiedGlobalConstantRule {
    type Config = NoFullyQualifiedGlobalConstantConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Fully Qualified Global Constant",
            code: "no-fully-qualified-global-constant",
            description: indoc! {"
                Disallows fully-qualified references to global constants within a namespace.

                Instead of using the backslash prefix (e.g., `\\PHP_VERSION`),
                prefer an explicit `use const` import statement. This improves
                readability and keeps imports centralized at the top of the file.
            "},
            good_example: indoc! {r#"
                <?php

                namespace App;

                use const PHP_VERSION;

                $version = PHP_VERSION;
            "#},
            bad_example: indoc! {r#"
                <?php

                namespace App;

                $version = \PHP_VERSION;
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

        if !identifier.is_fully_qualified() {
            return;
        }

        let constant_name = identifier.value().trim_start_matches('\\');

        // Skip true, false, null — these are language keywords.
        let name_lower = constant_name.to_ascii_lowercase();
        if matches!(name_lower.as_str(), "true" | "false" | "null") {
            return;
        }

        let short_name = constant_name.rsplit('\\').next().unwrap_or(constant_name);
        let fqn_span = identifier.span();

        let resolution = ctx.import_constant(constant_name);

        let (title, help) = match &resolution {
            Some(res) if res.is_already_available() && res.local_name.as_str() != short_name => (
                "Fully-qualified constant access can be replaced with an existing alias.",
                format!(
                    "`{constant_name}` is already imported as `{}`; replace the reference with it.",
                    res.local_name
                ),
            ),
            Some(res) if res.is_already_available() => (
                "Fully-qualified constant access is already in scope.",
                format!("`{constant_name}` is already reachable as `{}`; drop the leading `\\`.", res.local_name),
            ),
            Some(_) | None => (
                "Fully-qualified constant access detected.",
                format!("Add `use const {constant_name};` and reference `{short_name}` directly."),
            ),
        };

        let issue = Issue::new(self.cfg.level, title)
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(fqn_span)
                    .with_message(format!("The constant `\\{constant_name}` uses a fully-qualified name")),
            )
            .with_note("Fully-qualified constant access bypasses the import system, making it harder to see which global constants a file depends on.")
            .with_help(help);

        match resolution {
            Some(resolution) => {
                ctx.collector.propose(issue, |edits| {
                    edits.push(TextEdit::replace(fqn_span, resolution.local_name.as_str()));
                    if let Some(use_edit) = resolution.use_statement_edit {
                        edits.push(use_edit.with_safety(Safety::Safe));
                    }
                });
            }
            None => {
                ctx.collector.report(issue);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoFullyQualifiedGlobalConstantRule;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_success! {
        name = imported_constant_is_not_flagged,
        rule = NoFullyQualifiedGlobalConstantRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use const PHP_VERSION;

            $version = PHP_VERSION;
        "#}
    }

    test_lint_success! {
        name = global_scope_fq_constant_is_not_flagged,
        rule = NoFullyQualifiedGlobalConstantRule,
        code = indoc! {r#"
            <?php

            $version = \PHP_VERSION;
        "#}
    }

    test_lint_success! {
        name = fq_true_false_null_not_flagged,
        rule = NoFullyQualifiedGlobalConstantRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $a = \true;
            $b = \false;
            $c = \null;
        "#}
    }

    test_lint_failure! {
        name = fq_constant_access_in_namespace,
        rule = NoFullyQualifiedGlobalConstantRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $version = \PHP_VERSION;
        "#}
    }

    test_lint_fix! {
        name = fix_fq_constant_adds_use_const,
        rule = NoFullyQualifiedGlobalConstantRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $version = \PHP_VERSION;
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use const PHP_VERSION;

            $version = PHP_VERSION;
        "#}
    }

    test_lint_fix! {
        name = fix_three_fq_constants_in_one_pass,
        rule = NoFullyQualifiedGlobalConstantRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $a = \PHP_VERSION;
            $b = \PHP_EOL;
            $c = \PHP_INT_MAX;
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use const PHP_VERSION;

            use const PHP_EOL;

            use const PHP_INT_MAX;

            $a = PHP_VERSION;
            $b = PHP_EOL;
            $c = PHP_INT_MAX;
        "#}
    }

    test_lint_fix! {
        name = fix_many_references_to_same_fq_constant_one_use,
        rule = NoFullyQualifiedGlobalConstantRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $a = \PHP_EOL;
            $b = \PHP_EOL;
            $c = \PHP_EOL;
            $d = \PHP_EOL;
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use const PHP_EOL;

            $a = PHP_EOL;
            $b = PHP_EOL;
            $c = PHP_EOL;
            $d = PHP_EOL;
        "#}
    }

    test_lint_fix! {
        name = fix_fq_constant_appends_after_existing_use_const,
        rule = NoFullyQualifiedGlobalConstantRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use const PHP_EOL;

            $version = \PHP_VERSION;
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use const PHP_EOL;
            use const PHP_VERSION;

            $version = PHP_VERSION;
        "#}
    }

    test_lint_fix! {
        name = fix_fq_constant_appends_after_last_of_several_existing_uses,
        rule = NoFullyQualifiedGlobalConstantRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use const PHP_EOL;
            use const PHP_INT_MAX;
            use const PHP_INT_MIN;

            $version = \PHP_VERSION;
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use const PHP_EOL;
            use const PHP_INT_MAX;
            use const PHP_INT_MIN;
            use const PHP_VERSION;

            $version = PHP_VERSION;
        "#}
    }

    test_lint_failure! {
        name = fix_declined_when_short_name_conflicts_with_local_constant,
        rule = NoFullyQualifiedGlobalConstantRule,
        code = indoc! {r#"
            <?php

            namespace App;

            const PHP_VERSION = "9.0";

            $v = \PHP_VERSION;
        "#}
    }

    test_lint_failure! {
        name = fix_declined_when_short_name_conflicts_with_existing_use_const,
        rule = NoFullyQualifiedGlobalConstantRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use const Other\PHP_VERSION;

            $v = \PHP_VERSION;
        "#}
    }

    test_lint_fix! {
        name = fix_fq_constants_in_multiple_braced_namespaces_independent,
        rule = NoFullyQualifiedGlobalConstantRule,
        code = indoc! {r#"
            <?php

            namespace A {
                $x = \PHP_EOL;
            }

            namespace B {
                $y = \PHP_EOL;
            }
        "#},
        fixed = indoc! {r#"
            <?php

            namespace A {

            use const PHP_EOL;
                $x = PHP_EOL;
            }

            namespace B {

            use const PHP_EOL;
                $y = PHP_EOL;
            }
        "#}
    }

    test_lint_success! {
        name = fq_constant_via_existing_aliased_use_const_not_flagged_after_rewrite,
        rule = NoFullyQualifiedGlobalConstantRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use const PHP_VERSION as VERSION;

            $v = VERSION;
        "#}
    }

    test_lint_success! {
        name = short_reference_in_global_is_not_flagged,
        rule = NoFullyQualifiedGlobalConstantRule,
        code = indoc! {r#"
            <?php

            $v = PHP_VERSION;
        "#}
    }

    test_lint_success! {
        name = lowercase_true_false_null_in_namespace_not_flagged,
        rule = NoFullyQualifiedGlobalConstantRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $a = \True;
            $b = \FALSE;
            $c = \Null;
        "#}
    }

    test_lint_success! {
        name = fq_constant_in_nested_sub_namespace_unrelated_aliased_use,
        rule = NoFullyQualifiedGlobalConstantRule,
        code = indoc! {r#"
            <?php

            namespace App\Sub;

            use const Other\HELPER;

            $x = HELPER;
        "#}
    }
}
