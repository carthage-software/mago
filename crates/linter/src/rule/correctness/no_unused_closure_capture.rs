use foldhash::HashSet;
use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::ast::Closure;
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
pub struct NoUnusedClosureCaptureRule {
    meta: &'static RuleMeta,
    cfg: NoUnusedClosureCaptureConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoUnusedClosureCaptureConfig {
    pub level: Level,
}

impl Default for NoUnusedClosureCaptureConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoUnusedClosureCaptureConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoUnusedClosureCaptureRule {
    type Config = NoUnusedClosureCaptureConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Unused Closure Capture",
            code: "no-unused-closure-capture",
            description: indoc! {"
                Flags variables in a closure's `use (...)` clause that are never read
                or written inside the closure body.

                Captures only earn their keep when the body refers to them. A capture
                that nothing observes is usually a leftover from a refactor or a typo
                in the captured name.

                Names beginning with an underscore (`$_`, `$_foo`) are treated as
                intentionally-discarded and are ignored. By-reference captures
                (`use (&$x)`) are also ignored — they are commonly used for their
                side-effect on the outer scope, even when the inner body doesn't
                otherwise touch the binding. The rule bails out of any closure body
                that uses variable variables (`$$x`, `${expr}`) or calls `extract()`.
            "},
            good_example: indoc! {r#"
                <?php

                $base = 10;
                $add = function (int $x) use ($base): int {
                    return $x + $base;
                };
            "#},
            bad_example: indoc! {r#"
                <?php

                $base = 10;
                $add = function (int $x) use ($base): int {
                    return $x;
                };
            "#},
            category: Category::Correctness,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Closure];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Closure(Closure { use_clause: Some(use_clause), body, .. }) = node else {
            return;
        };

        let captures: Vec<_> = use_clause
            .variables
            .iter()
            .filter(|cap| cap.ampersand.is_none())
            .filter(|cap| !variable_usage::is_silenced_name(cap.variable.name))
            .collect();
        if captures.is_empty() {
            return;
        }

        let interest: HashSet<&'arena str> = captures.iter().map(|cap| cap.variable.name).collect();
        let usage = variable_usage::collect_used_names(body, interest);
        if usage.bailed {
            return;
        }

        for cap in captures {
            let name = cap.variable.name;
            if usage.referenced.contains(name) {
                continue;
            }

            ctx.collector.report(
                Issue::new(self.cfg.level(), format!("Closure capture `{name}` is unused."))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(cap.variable.span)
                            .with_message(format!("`{name}` is captured here but never read in the closure body")),
                    )
                    .with_note("If you only need to mutate the outer binding, use `&` to capture by reference.")
                    .with_help("Remove this capture, or reference it inside the closure body."),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoUnusedClosureCaptureRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = simple_unused_capture,
        rule = NoUnusedClosureCaptureRule,
        code = indoc! {r#"
            <?php

            $base = 10;
            $cb = function () use ($base): int {
                return 0;
            };
        "#}
    }

    test_lint_failure! {
        name = multiple_captures_one_unused,
        rule = NoUnusedClosureCaptureRule,
        count = 1,
        code = indoc! {r#"
            <?php

            $a = 1;
            $b = 2;
            $cb = function () use ($a, $b): int {
                return $a;
            };
        "#}
    }

    test_lint_success! {
        name = capture_used_for_read,
        rule = NoUnusedClosureCaptureRule,
        code = indoc! {r#"
            <?php

            $base = 10;
            $cb = function (int $x) use ($base): int {
                return $x + $base;
            };
        "#}
    }

    test_lint_success! {
        name = capture_used_inside_arrow_function,
        rule = NoUnusedClosureCaptureRule,
        code = indoc! {r#"
            <?php

            $base = 10;
            $cb = function () use ($base): Closure {
                return fn(int $x) => $x + $base;
            };
        "#}
    }

    test_lint_success! {
        name = capture_used_inside_inner_closure_use_clause,
        rule = NoUnusedClosureCaptureRule,
        code = indoc! {r#"
            <?php

            $base = 10;
            $cb = function () use ($base): Closure {
                return function () use ($base): int {
                    return $base;
                };
            };
        "#}
    }

    test_lint_success! {
        name = by_reference_capture_ignored_even_if_unused_in_body,
        rule = NoUnusedClosureCaptureRule,
        code = indoc! {r#"
            <?php

            $sink = null;
            $cb = function () use (&$sink): void {
            };
        "#}
    }

    test_lint_success! {
        name = underscore_capture_silenced,
        rule = NoUnusedClosureCaptureRule,
        code = indoc! {r#"
            <?php

            $_unused = 1;
            $cb = function () use ($_unused): int {
                return 0;
            };
        "#}
    }

    test_lint_success! {
        name = capture_used_via_compact,
        rule = NoUnusedClosureCaptureRule,
        code = indoc! {r#"
            <?php

            $name = 'mago';
            $cb = function () use ($name): array {
                return compact('name');
            };
        "#}
    }

    test_lint_success! {
        name = extract_bails,
        rule = NoUnusedClosureCaptureRule,
        code = indoc! {r#"
            <?php

            $config = ['foo' => 1];
            $cb = function (array $data) use ($config): void {
                extract($data);
            };
        "#}
    }

    test_lint_success! {
        name = variable_variable_bails,
        rule = NoUnusedClosureCaptureRule,
        code = indoc! {r#"
            <?php

            $secret = 'shh';
            $cb = function (string $name) use ($secret): void {
                $$name = 1;
            };
        "#}
    }

    test_lint_success! {
        name = capture_written_only_still_used,
        rule = NoUnusedClosureCaptureRule,
        code = indoc! {r#"
            <?php

            $config = [];
            $cb = function () use ($config): void {
                $config = ['reset'];
            };
        "#}
    }

    test_lint_success! {
        name = closure_without_use_clause_ignored,
        rule = NoUnusedClosureCaptureRule,
        code = indoc! {r#"
            <?php

            $cb = function (int $x): int {
                return $x + 1;
            };
        "#}
    }
}
