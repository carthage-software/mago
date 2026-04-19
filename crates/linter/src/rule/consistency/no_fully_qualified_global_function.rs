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
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionCall, NodeKind::FunctionPartialApplication];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        if ctx.scope.get_namespace().is_empty() {
            return;
        }

        let identifier = match node {
            Node::FunctionCall(call) => {
                let Expression::Identifier(identifier) = call.function else {
                    return;
                };
                identifier
            }
            Node::FunctionPartialApplication(application) => {
                let Expression::Identifier(identifier) = application.function else {
                    return;
                };
                identifier
            }
            _ => return,
        };

        if !identifier.is_fully_qualified() {
            return;
        }

        let function_name = identifier.value().trim_start_matches('\\');
        let short_name = function_name.rsplit('\\').next().unwrap_or(function_name);
        let fqn_span = identifier.span();

        let resolution = ctx.import_function(function_name);

        let (title, help) = match &resolution {
            Some(res) if res.is_already_available() && res.local_name.as_str() != short_name => (
                "Fully-qualified function call can be replaced with an existing alias.",
                format!("`{function_name}` is already imported as `{}`; replace the call with it.", res.local_name),
            ),
            Some(res) if res.is_already_available() => (
                "Fully-qualified function call is already in scope.",
                format!("`{function_name}` is already reachable as `{}`; drop the leading `\\`.", res.local_name),
            ),
            Some(_) | None => (
                "Fully-qualified function call detected.",
                format!("Add `use function {function_name};` and call `{short_name}(...)` directly."),
            ),
        };

        let issue = Issue::new(self.cfg.level, title)
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(fqn_span)
                    .with_message(format!("The call to `\\{function_name}` uses a fully-qualified name")),
            )
            .with_note("Fully-qualified function calls bypass the import system, making it harder to see which global functions a file depends on.")
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

    use super::NoFullyQualifiedGlobalFunctionRule;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
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

    test_lint_failure! {
        name = fq_function_partial_application_in_namespace,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $fn = \strlen(...);
        "#}
    }

    test_lint_fix! {
        name = fix_fq_function_adds_use_function,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $length = \strlen("hello");
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use function strlen;

            $length = strlen("hello");
        "#}
    }

    test_lint_fix! {
        name = fix_fq_function_appends_after_existing_use_function,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use function array_map;

            $length = \strlen("hello");
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use function array_map;
            use function strlen;

            $length = strlen("hello");
        "#}
    }

    test_lint_fix! {
        name = fix_three_fq_functions_in_one_pass,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $a = \strlen("a");
            $b = \strtoupper("b");
            $c = \trim("c");
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use function strlen;

            use function strtoupper;

            use function trim;

            $a = strlen("a");
            $b = strtoupper("b");
            $c = trim("c");
        "#}
    }

    test_lint_fix! {
        name = fix_many_calls_to_same_fq_function_one_use,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $a = \strlen("a");
            $b = \strlen("b");
            $c = \strlen("c");
            $d = \strlen("d");
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use function strlen;

            $a = strlen("a");
            $b = strlen("b");
            $c = strlen("c");
            $d = strlen("d");
        "#}
    }

    test_lint_failure! {
        name = fix_declined_when_short_name_conflicts_with_local_function,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            function strlen(string $s): int { return 0; }

            $a = \strlen("a");
        "#}
    }

    test_lint_failure! {
        name = fix_declined_when_short_name_conflicts_with_existing_use_function,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use function Other\strlen;

            $a = \strlen("a");
        "#}
    }

    test_lint_fix! {
        name = fix_fq_function_appends_after_last_existing_use_function,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use function array_map;
            use function array_filter;
            use function array_reduce;

            $length = \strlen("hello");
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use function array_map;
            use function array_filter;
            use function array_reduce;
            use function strlen;

            $length = strlen("hello");
        "#}
    }

    test_lint_fix! {
        name = fix_fq_function_in_multiple_braced_namespaces_independent,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace A {
                $x = \strlen("a");
            }

            namespace B {
                $y = \strlen("b");
            }
        "#},
        fixed = indoc! {r#"
            <?php

            namespace A {

            use function strlen;
                $x = strlen("a");
            }

            namespace B {

            use function strlen;
                $y = strlen("b");
            }
        "#}
    }

    test_lint_success! {
        name = fq_function_via_existing_aliased_use_is_not_flagged_after_rewrite,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use function strlen as length;

            $n = length("hello");
        "#}
    }

    test_lint_failure! {
        name = fq_function_with_existing_alias_for_different_name_still_flagged_but_conflict_blocks_fix,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use function Other\something as strlen;

            $n = \strlen("hello");
        "#}
    }

    test_lint_success! {
        name = short_call_in_global_is_not_flagged,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            $n = strlen("x");
        "#}
    }

    test_lint_failure! {
        name = fq_function_partial_application_in_namespace_flagged,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $fn = \strlen(...);
        "#}
    }

    test_lint_fix! {
        name = fix_fq_function_partial_application_adds_use,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $fn = \strlen(...);
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use function strlen;

            $fn = strlen(...);
        "#}
    }

    test_lint_success! {
        name = fq_function_in_nested_sub_namespace_with_matching_current_is_handled,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App\Sub;

            use function Other\helper;

            $x = helper();
        "#}
    }
}
