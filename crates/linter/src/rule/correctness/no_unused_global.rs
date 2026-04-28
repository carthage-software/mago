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
pub struct NoUnusedGlobalRule {
    meta: &'static RuleMeta,
    cfg: NoUnusedGlobalConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoUnusedGlobalConfig {
    pub level: Level,
}

impl Default for NoUnusedGlobalConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoUnusedGlobalConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoUnusedGlobalRule {
    type Config = NoUnusedGlobalConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Unused Global",
            code: "no-unused-global",
            description: indoc! {"
                Flags `global $x;` declarations whose name is never read or written
                inside the surrounding function-like scope.

                A `global` statement only earns its keep when later code refers to the
                imported binding. If the name is never used, the statement is dead —
                usually a leftover from a refactor or a typo in the imported name.

                Names beginning with an underscore (`$_`, `$_foo`) are treated as
                intentionally-discarded and are ignored. The rule bails out of any
                scope that uses variable variables (`$$x`, `${expr}`) or calls
                `extract()`, since those introduce names the linter cannot resolve.
            "},
            good_example: indoc! {r#"
                <?php

                function bump(): void {
                    global $counter;
                    $counter++;
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                function f(): void {
                    global $forgotten;
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

        let mut decls = GlobalDeclCollector { items: Vec::new() };
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
                Issue::new(self.cfg.level(), format!("Global variable `{name}` is imported but never used."))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(*span)
                            .with_message(format!("`{name}` is imported via `global` here but never read or written")),
                    )
                    .with_help("Remove this `global` declaration, or reference the variable below."),
            );
        }
    }
}

struct GlobalDeclCollector<'arena> {
    items: Vec<(&'arena str, mago_span::Span)>,
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for GlobalDeclCollector<'arena> {
    fn walk_global(&mut self, g: &'ast Global<'arena>, _: &mut ()) {
        for v in g.variables.iter() {
            if let Variable::Direct(d) = v {
                self.items.push((d.name, d.span));
            }
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

    use super::NoUnusedGlobalRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = simple_unused_global,
        rule = NoUnusedGlobalRule,
        code = indoc! {r#"
            <?php

            function t(): void {
                global $forgottenthing;
            }
        "#}
    }

    test_lint_failure! {
        name = multiple_globals_some_unused,
        rule = NoUnusedGlobalRule,
        count = 1,
        code = indoc! {r#"
            <?php

            function f(): int {
                global $used, $unused;
                return $used;
            }
        "#}
    }

    test_lint_failure! {
        name = unused_global_in_closure,
        rule = NoUnusedGlobalRule,
        code = indoc! {r#"
            <?php

            $cb = function (): void {
                global $forgotten;
            };
        "#}
    }

    test_lint_failure! {
        name = unused_global_in_method,
        rule = NoUnusedGlobalRule,
        code = indoc! {r#"
            <?php

            class C {
                public function f(): void {
                    global $never;
                }
            }
        "#}
    }

    test_lint_failure! {
        name = global_only_referenced_inside_inner_function_definition_does_not_count,
        rule = NoUnusedGlobalRule,
        code = indoc! {r#"
            <?php

            function outer(): void {
                global $config;

                function inner(): void {
                    echo $config; // separate scope, does not satisfy outer global
                }
            }
        "#}
    }

    test_lint_success! {
        name = global_used_for_read,
        rule = NoUnusedGlobalRule,
        code = indoc! {r#"
            <?php

            function read(): mixed {
                global $config;
                return $config;
            }
        "#}
    }

    test_lint_success! {
        name = global_used_for_write,
        rule = NoUnusedGlobalRule,
        code = indoc! {r#"
            <?php

            function bump(): void {
                global $counter;
                $counter++;
            }
        "#}
    }

    test_lint_success! {
        name = global_used_in_compact,
        rule = NoUnusedGlobalRule,
        code = indoc! {r#"
            <?php

            function snapshot(): array {
                global $config;
                return compact('config');
            }
        "#}
    }

    test_lint_success! {
        name = global_captured_in_inner_closure_use_clause,
        rule = NoUnusedGlobalRule,
        code = indoc! {r#"
            <?php

            function lazy(): Closure {
                global $config;
                return function () use ($config) {
                    return $config;
                };
            }
        "#}
    }

    test_lint_success! {
        name = global_used_in_arrow_function,
        rule = NoUnusedGlobalRule,
        code = indoc! {r#"
            <?php

            function lazy(): Closure {
                global $config;
                return fn() => $config;
            }
        "#}
    }

    test_lint_success! {
        name = global_underscore_silenced,
        rule = NoUnusedGlobalRule,
        code = indoc! {r#"
            <?php

            function f(): void {
                global $_unused;
            }
        "#}
    }

    test_lint_success! {
        name = extract_bails,
        rule = NoUnusedGlobalRule,
        code = indoc! {r#"
            <?php

            function f(array $data): void {
                global $config;
                extract($data);
            }
        "#}
    }

    test_lint_success! {
        name = variable_variable_bails,
        rule = NoUnusedGlobalRule,
        code = indoc! {r#"
            <?php

            function f(): void {
                global $config;
                $name = 'config';
                $$name = 1;
            }
        "#}
    }

    test_lint_success! {
        name = global_unset_counts_as_use,
        rule = NoUnusedGlobalRule,
        code = indoc! {r#"
            <?php

            function reset_config(): void {
                global $config;
                unset($config);
            }
        "#}
    }
}
