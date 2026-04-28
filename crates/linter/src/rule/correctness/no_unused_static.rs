use foldhash::HashSet;
use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::ast::*;
use mago_syntax::walker::MutWalker;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::variable_usage;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoUnusedStaticRule {
    meta: &'static RuleMeta,
    cfg: NoUnusedStaticConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoUnusedStaticConfig {
    pub level: Level,
}

impl Default for NoUnusedStaticConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoUnusedStaticConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoUnusedStaticRule {
    type Config = NoUnusedStaticConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Unused Static",
            code: "no-unused-static",
            description: indoc! {"
                Flags `static $x;` declarations whose name is never read or written
                inside the surrounding function-like scope.

                A `static` declaration only earns its keep when later code refers to
                the binding. If the name is never used after the declaration, the
                statement is dead — usually a leftover from a refactor.

                Names beginning with an underscore (`$_`, `$_foo`) are treated as
                intentionally-discarded and are ignored. The rule bails out of any
                scope that uses variable variables (`$$x`, `${expr}`) or calls
                `extract()`, since those introduce names the linter cannot resolve.
            "},
            good_example: indoc! {r#"
                <?php

                function next_id(): int {
                    static $counter = 0;
                    return ++$counter;
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                function f(): void {
                    static $forgotten = 0;
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
        let Some((_, body, _)) = variable_usage::function_like_parts(node) else {
            return;
        };

        let mut decls = StaticDeclCollector { items: Vec::new() };
        decls.walk_block(body, &mut ());
        if decls.items.is_empty() {
            return;
        }

        let interest: HashSet<&'arena str> = decls.items.iter().map(|(name, _)| *name).collect();
        let usage = variable_usage::collect_used_names(body, interest);
        if usage.bailed {
            return;
        }

        for (name, span) in &decls.items {
            if variable_usage::is_silenced_name(name) {
                continue;
            }

            if usage.referenced.contains(name) {
                continue;
            }

            ctx.collector.report(
                Issue::new(self.cfg.level(), format!("Static variable `{name}` is declared but never used."))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(*span)
                            .with_message(format!("`{name}` is declared `static` here but never read or written")),
                    )
                    .with_help("Remove this `static` declaration, or reference the variable below."),
            );
        }
    }
}

struct StaticDeclCollector<'arena> {
    items: Vec<(&'arena str, mago_span::Span)>,
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for StaticDeclCollector<'arena> {
    fn walk_static(&mut self, s: &'ast Static<'arena>, _: &mut ()) {
        for item in s.items.iter() {
            let var = item.variable();
            self.items.push((var.name, var.span));
        }
    }

    fn walk_function(&mut self, _: &'ast Function<'arena>, _: &mut ()) {}
    fn walk_method(&mut self, _: &'ast Method<'arena>, _: &mut ()) {}
    fn walk_closure(&mut self, _: &'ast Closure<'arena>, _: &mut ()) {}
    fn walk_arrow_function(&mut self, _: &'ast ArrowFunction<'arena>, _: &mut ()) {}
    fn walk_class(&mut self, _: &'ast Class<'arena>, _: &mut ()) {}
    fn walk_interface(&mut self, _: &'ast Interface<'arena>, _: &mut ()) {}
    fn walk_trait(&mut self, _: &'ast Trait<'arena>, _: &mut ()) {}
    fn walk_enum(&mut self, _: &'ast Enum<'arena>, _: &mut ()) {}
    fn walk_anonymous_class(&mut self, _: &'ast AnonymousClass<'arena>, _: &mut ()) {}
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoUnusedStaticRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = simple_unused_static,
        rule = NoUnusedStaticRule,
        code = indoc! {r#"
            <?php

            function f(): void {
                static $forgotten;
            }
        "#}
    }

    test_lint_failure! {
        name = unused_static_with_initializer,
        rule = NoUnusedStaticRule,
        code = indoc! {r#"
            <?php

            function f(): void {
                static $counter = 0;
            }
        "#}
    }

    test_lint_failure! {
        name = multiple_in_one_static_some_unused,
        rule = NoUnusedStaticRule,
        count = 1,
        code = indoc! {r#"
            <?php

            function f(): int {
                static $used = 0, $unused = 0;
                return ++$used;
            }
        "#}
    }

    test_lint_failure! {
        name = static_unused_in_closure,
        rule = NoUnusedStaticRule,
        code = indoc! {r#"
            <?php

            $cb = function (): void {
                static $forgotten = 0;
            };
        "#}
    }

    test_lint_failure! {
        name = static_unused_in_method,
        rule = NoUnusedStaticRule,
        code = indoc! {r#"
            <?php

            class C {
                public function tick(): void {
                    static $never_touched = 0;
                }
            }
        "#}
    }

    test_lint_success! {
        name = static_used_via_increment,
        rule = NoUnusedStaticRule,
        code = indoc! {r#"
            <?php

            function next_id(): int {
                static $counter = 0;
                return ++$counter;
            }
        "#}
    }

    test_lint_success! {
        name = static_used_in_compact,
        rule = NoUnusedStaticRule,
        code = indoc! {r#"
            <?php

            function snapshot(): array {
                static $latest = null;
                return compact('latest');
            }
        "#}
    }

    test_lint_success! {
        name = static_used_inside_nested_closure_use_clause,
        rule = NoUnusedStaticRule,
        code = indoc! {r#"
            <?php

            function next(): Closure {
                static $counter = 0;
                return function () use ($counter): int {
                    return $counter;
                };
            }
        "#}
    }

    test_lint_success! {
        name = static_used_inside_arrow_function,
        rule = NoUnusedStaticRule,
        code = indoc! {r#"
            <?php

            function next(): Closure {
                static $counter = 0;
                return fn() => $counter;
            }
        "#}
    }

    test_lint_success! {
        name = static_underscore_silenced,
        rule = NoUnusedStaticRule,
        code = indoc! {r#"
            <?php

            function f(): void {
                static $_unused = 0;
            }
        "#}
    }

    test_lint_success! {
        name = extract_bails,
        rule = NoUnusedStaticRule,
        code = indoc! {r#"
            <?php

            function f(array $data): void {
                static $secret = 'shh';
                extract($data);
            }
        "#}
    }

    test_lint_success! {
        name = variable_variable_bails,
        rule = NoUnusedStaticRule,
        code = indoc! {r#"
            <?php

            function f(): void {
                static $secret = 'shh';
                $name = 'thing';
                $$name = 1;
            }
        "#}
    }

    test_lint_success! {
        name = static_written_only_still_used,
        rule = NoUnusedStaticRule,
        code = indoc! {r#"
            <?php

            function f(): void {
                static $log = [];
                $log[] = 'event';
            }
        "#}
    }
}
