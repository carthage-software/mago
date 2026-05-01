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
use crate::rule::utils::variable_usage::RedundantRecorder;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoRedundantVariableRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantVariableConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantVariableConfig {
    pub level: Level,
}

impl Default for NoRedundantVariableConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoRedundantVariableConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantVariableRule {
    type Config = NoRedundantVariableConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Variable",
            code: "no-redundant-variable",
            description: indoc! {"
                Flags variables that are written or declared but whose value is never read.

                Detects fully-unused variables (assigned and never referenced) as well as
                variables whose only mention is on the write side — for example, an
                undefined name passed to a function as a potential by-reference output where
                the result is never observed by the caller.

                Variables whose name starts with an underscore (`$_`, `$_foo`) are treated as
                intentionally-discarded and are ignored. Variables declared via `global` or
                `static` are also ignored, since they are bindings to external scope.

                The rule analyses one function-like scope at a time. It bails out of any scope
                that uses variable variables (`$$x`, `${expr}`) or calls `extract()`, since
                those introduce names the linter cannot resolve.
            "},
            good_example: indoc! {r#"
                <?php

                function greet(string $name): string
                {
                    $greeting = "Hello, $name!";

                    return $greeting;
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                function greet(string $name): string
                {
                    $unused = compute_something();

                    return "Hello, $name!";
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

        let usage: RedundantRecorder<'_> = variable_usage::analyze(parameter_list, body, use_clause);
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

            let Some(span) = info.pending_write else {
                continue;
            };

            let bare = name.strip_prefix('$').unwrap_or(name);
            ctx.collector.report(
                Issue::new(self.cfg.level(), format!("Variable `{name}` is assigned but never used."))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(span)
                            .with_message(format!("`{name}` is written here but its value is never observed")),
                    )
                    .with_note(
                        "If this is intentional (e.g. a discarded by-reference output), rename to `$_` or `$_<name>`.",
                    )
                    .with_help(format!(
                        "Remove the assignment, or rename the variable to `$_{bare}` to silence this rule."
                    )),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoRedundantVariableRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = simple_unused_variable,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f() {
                $a = 1;
                return;
            }
        "#}
    }

    test_lint_failure! {
        name = unused_assignment_with_side_effect_rhs,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f() {
                $name = compute_something();
                return 'world';
            }
        "#}
    }

    test_lint_failure! {
        name = potential_by_ref_assignment_unused,
        rule = NoRedundantVariableRule,
        count = 2,
        code = indoc! {r#"
            <?php

            function f() {
                $a = bar($y);
            }
        "#}
    }

    test_lint_success! {
        name = used_variable_is_fine,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f(string $name) {
                $greeting = "Hello, $name!";
                return $greeting;
            }
        "#}
    }

    test_lint_success! {
        name = underscore_prefix_silenced,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f() {
                $_ = compute();
                $_ignored = compute2();
            }
        "#}
    }

    test_lint_success! {
        name = global_declaration_kept,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f() {
                global $config;
            }
        "#}
    }

    test_lint_success! {
        name = static_declaration_kept,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function next_id(): int {
                static $counter = 0;
                $counter++;
                return $counter;
            }
        "#}
    }

    test_lint_success! {
        name = compact_marks_named_locals_as_used,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f() {
                $name = 'world';
                $age = 42;
                return compact('name', 'age');
            }
        "#}
    }

    test_lint_success! {
        name = extract_bails_function,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f(array $data) {
                $local = 1;
                extract($data);
                return $local;
            }
        "#}
    }

    test_lint_success! {
        name = variable_variable_bails_function,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f() {
                $name = 'foo';
                $$name = 1;
                return null;
            }
        "#}
    }

    test_lint_success! {
        name = if_else_branches_share_scope,
        rule = NoRedundantVariableRule,
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
        name = closure_use_clause_captures_outer,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f() {
                $base = 10;
                $add = function (int $x) use ($base) {
                    return $x + $base;
                };
                return $add(5);
            }
        "#}
    }

    test_lint_success! {
        name = arrow_fn_captures_outer_variable,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f() {
                $base = 10;
                $add = fn(int $x) => $x + $base;
                return $add(5);
            }
        "#}
    }

    test_lint_failure! {
        name = catch_variable_unused,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f() {
                try {
                    return run();
                } catch (Throwable $e) {
                    return null;
                }
            }
        "#}
    }

    test_lint_failure! {
        name = foreach_value_unused,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f(array $items) {
                $count = 0;
                foreach ($items as $item) {
                    $count++;
                }
                return $count;
            }
        "#}
    }

    test_lint_success! {
        name = unset_counts_as_use,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f() {
                $tmp = compute();
                unset($tmp);
            }
        "#}
    }

    test_lint_success! {
        name = foreach_loop_wraparound_read,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f(array $items): void {
                $shown = false;
                foreach ($items as $item) {
                    if (!$shown) {
                        echo "header\n";
                        $shown = true;
                    }
                    echo $item, "\n";
                }
            }
        "#}
    }

    test_lint_success! {
        name = while_loop_wraparound_read,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f(): void {
                $i = 0;
                $seen = false;
                while ($i < 10) {
                    if (!$seen) {
                        echo "first\n";
                        $seen = true;
                    }
                    $i++;
                }
            }
        "#}
    }

    test_lint_success! {
        name = do_while_loop_wraparound_read,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f(): void {
                $seen = false;
                do {
                    if (!$seen) {
                        echo "first\n";
                        $seen = true;
                    }
                } while (random_int(0, 1));
            }
        "#}
    }

    test_lint_success! {
        name = for_loop_wraparound_read,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f(): void {
                $seen = false;
                for ($i = 0; $i < 10; $i++) {
                    if (!$seen) {
                        echo "first\n";
                        $seen = true;
                    }
                }
            }
        "#}
    }

    test_lint_failure! {
        name = unused_variable_inside_loop,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f(array $items): void {
                foreach ($items as $item) {
                    $unused = 1;
                    echo $item, "\n";
                }
            }
        "#}
    }

    test_lint_failure! {
        name = body_writes_only_value_never_read,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            function f(array $items): void {
                foreach ($items as $item) {
                    $tmp = compute();
                    echo $item;
                }
            }
        "#}
    }

    test_lint_success! {
        name = nested_loop_running_totals_observed_on_next_iteration,
        rule = NoRedundantVariableRule,
        code = indoc! {r#"
            <?php

            /**
             * @param array<string, array<int, int>> $uploads
             */
            function tally(array $uploads, int $left_files, int $left_bytes): void {
                $files = 0;
                $bytes = 0;

                foreach ($uploads as $list) {
                    foreach ($list as $size) {
                        if ($files >= $left_files) {
                            continue;
                        }
                        $b = (int) $size;
                        if (($b + $bytes) > $left_bytes) {
                            continue;
                        }
                        ++$files;
                        $bytes += $b;
                    }
                }
            }
        "#}
    }
}
