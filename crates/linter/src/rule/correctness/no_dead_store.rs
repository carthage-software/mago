use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::variable_usage;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoDeadStoreRule {
    meta: &'static RuleMeta,
    cfg: NoDeadStoreConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoDeadStoreConfig {
    pub level: Level,
}

impl Default for NoDeadStoreConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoDeadStoreConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoDeadStoreRule {
    type Config = NoDeadStoreConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Dead Store",
            code: "no-dead-store",
            description: indoc! {"
                Flags assignments to a variable whose value is overwritten by a later
                assignment without being read in between. The earlier assignment is dead;
                its value never reaches anything observable.

                Detection is limited to linear (non-branching) flow. Writes inside conditional
                branches (if/else, loops, match arms, try paths, switch cases) don't pair up
                with writes in sibling branches, so this rule produces no false positives for
                code like `if ($cond) { $x = 1; } else { $x = 2; } return $x;`.

                Variables whose name starts with an underscore (`$_`, `$_foo`) are ignored.
                Variables declared via `global` or `static` are also ignored.

                The rule analyses one function-like scope at a time. It bails out of any scope
                that uses variable variables (`$$x`, `${expr}`) or calls `extract()`.
            "},
            good_example: indoc! {r#"
                <?php

                function f() {
                    $x = compute();
                    return $x;
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                function f() {
                    $x = 1; // dead - overwritten before being read
                    $x = compute();
                    return $x;
                }
            "#},
            category: Category::Correctness,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Function, NodeKind::Method, NodeKind::Closure];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Some((parameter_list, body, use_clause)) = variable_usage::function_like_parts(node) else {
            return;
        };

        let usage: variable_usage::DeadStoreRecorder<'_> = variable_usage::analyze(parameter_list, body, use_clause);
        if usage.bailed {
            return;
        }

        for (name, info) in &usage.info {
            if info.do_not_flag {
                continue;
            }

            if variable_usage::is_silenced_name(name) {
                continue;
            }

            for span in &info.dead_stores {
                ctx.collector.report(
                    Issue::new(self.cfg.level(), format!("Variable `{name}` is overwritten before its value is read."))
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(*span)
                                .with_message(format!("`{name}` is assigned here but overwritten without being read")),
                        )
                        .with_help("Remove this assignment, or use the value before the next assignment."),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoDeadStoreRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = simple_dead_store,
        rule = NoDeadStoreRule,
        code = indoc! {r#"
            <?php

            function f() {
                $x = 1;
                $x = 2;
                return $x;
            }
        "#}
    }

    test_lint_failure! {
        name = three_assignments_first_two_dead,
        rule = NoDeadStoreRule,
        count = 2,
        code = indoc! {r#"
            <?php

            function f() {
                $x = 1;
                $x = 2;
                $x = 3;
                return $x;
            }
        "#}
    }

    test_lint_success! {
        name = compound_assignment_uses_previous_value,
        rule = NoDeadStoreRule,
        code = indoc! {r#"
            <?php

            function f(int $x) {
                $sum = $x;
                $sum += 1;
                return $sum;
            }
        "#}
    }

    test_lint_success! {
        name = if_else_branches_do_not_pair,
        rule = NoDeadStoreRule,
        code = indoc! {r#"
            <?php

            function f(bool $cond) {
                if ($cond) {
                    $msg = 'yes';
                } else {
                    $msg = 'no';
                }
                return $msg;
            }
        "#}
    }

    test_lint_success! {
        name = no_dead_store_when_value_is_used,
        rule = NoDeadStoreRule,
        code = indoc! {r#"
            <?php

            function f() {
                $x = 1;
                echo $x;
                $x = 2;
                return $x;
            }
        "#}
    }

    test_lint_success! {
        name = single_unused_assignment_is_not_a_dead_store,
        rule = NoDeadStoreRule,
        code = indoc! {r#"
            <?php

            function f() {
                $x = 1;
                return;
            }
        "#}
    }

    test_lint_success! {
        name = underscore_prefix_silenced,
        rule = NoDeadStoreRule,
        code = indoc! {r#"
            <?php

            function f() {
                $_ = 1;
                $_ = 2;
                return;
            }
        "#}
    }

    test_lint_success! {
        name = extract_bails_function,
        rule = NoDeadStoreRule,
        code = indoc! {r#"
            <?php

            function f(array $data) {
                $local = 1;
                $local = 2;
                extract($data);
                return $local;
            }
        "#}
    }

    test_lint_failure! {
        name = dead_store_inside_branch,
        rule = NoDeadStoreRule,
        code = indoc! {r#"
            <?php

            function f(bool $cond) {
                if ($cond) {
                    $x = 1;
                    $x = 2;
                    return $x;
                }
                return null;
            }
        "#}
    }

    test_lint_success! {
        name = foreach_target_does_not_dead_store_pre_loop,
        rule = NoDeadStoreRule,
        code = indoc! {r#"
            <?php

            function some(mixed $_): void {}

            /** @param array<string|null> $a */
            function t(array $a): void {
                $child = 0;
                foreach ($a as $child) {
                    some($child);
                }
                some($child);
            }
        "#}
    }

    test_lint_success! {
        name = foreach_key_value_target_does_not_dead_store_pre_loop,
        rule = NoDeadStoreRule,
        code = indoc! {r#"
            <?php

            function some(mixed $_): void {}

            function t(array $a): void {
                $k = 'pre';
                $v = 'pre';
                foreach ($a as $k => $v) {
                    some($k);
                    some($v);
                }
                some($k);
                some($v);
            }
        "#}
    }
}
