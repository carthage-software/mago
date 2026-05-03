use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Block;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::*;
use mago_syntax::walker::MutWalker;
use mago_syntax::walker::walk_block_mut;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const DEFAULT_THRESHOLD: usize = 7;

#[derive(Debug, Clone)]
pub struct ExcessiveNestingRule {
    meta: &'static RuleMeta,
    cfg: ExcessiveNestingConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ExcessiveNestingConfig {
    pub level: Level,
    pub threshold: usize,
    /// Maximum nesting depth allowed inside a single function, method, closure, or arrow function.
    ///
    /// When set, each function-like body is checked independently against this threshold,
    /// with nesting counted from the function body (not the file root).
    ///
    /// Default: `None` (function-like bodies are only checked against the global `threshold`).
    pub function_like_threshold: Option<usize>,
}

impl Default for ExcessiveNestingConfig {
    fn default() -> Self {
        Self { level: Level::Warning, threshold: DEFAULT_THRESHOLD, function_like_threshold: None }
    }
}

impl Config for ExcessiveNestingConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ExcessiveNestingRule {
    type Config = ExcessiveNestingConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Excessive Nesting",
            code: "excessive-nesting",
            description: indoc! {r"
                Checks if the nesting level in any block exceeds a configurable threshold.

                Deeply nested code is harder to read, understand, and maintain.
                Consider refactoring to use early returns, helper methods, or clearer control flow.

                The `function-like-threshold` option allows setting a separate, typically lower,
                threshold for individual functions, methods, closures, and property hooks.
            "},
            good_example: indoc! {r#"
                <?php

                if ($condition) {
                    while ($otherCondition) {
                        echo "Hello"; // nesting depth = 2
                    }
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                if ($a) {
                    if ($b) {
                        if ($c) {
                            if ($d) {
                                if ($e) {
                                    if ($f) {
                                        if ($g) {
                                            if ($h) {
                                                echo "Too deeply nested!";
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            "#},
            category: Category::Maintainability,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Program];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        let mut walker = NestingWalker {
            effective_threshold: self.cfg.threshold,
            function_like_threshold: self.cfg.function_like_threshold,
            level: 0,
            meta: self.meta,
            cfg: self.cfg,
            function_like: false,
        };

        walker.walk_program(program, ctx);
    }
}

struct NestingWalker {
    effective_threshold: usize,
    function_like_threshold: Option<usize>,
    level: usize,
    meta: &'static RuleMeta,
    cfg: ExcessiveNestingConfig,
    function_like: bool,
}

impl NestingWalker {
    fn check_depth(&self, block: &Block, ctx: &mut LintContext) -> bool {
        let threshold = self.effective_threshold;
        if self.level > threshold {
            let scope = if self.function_like { "function-like" } else { "global" };

            ctx.collector.report(
                Issue::new(self.cfg.level, "Excessive block nesting.")
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(block.span()).with_message(format!(
                        "This block has a nesting depth of {}, which exceeds the {scope} threshold of {threshold}.",
                        self.level,
                    )))
                    .with_note(format!(
                        "The {scope} nesting level is {}, which is greater than the allowed {scope} threshold of {threshold}.",
                        self.level,
                    ))
                    .with_note("Excessive nesting can make code harder to read, understand, and maintain.")
                    .with_help(
                        "Refactor your code to reduce nesting (e.g. use early returns, guard clauses, or helper functions).",
                    ),
            );

            return true;
        }

        false
    }

    fn enter_function_like_body<'arena>(&mut self, body: &Block<'arena>, ctx: &mut LintContext<'_, 'arena>) {
        let Some(fn_threshold) = self.function_like_threshold else {
            return;
        };

        let saved_function_like = self.function_like;
        let saved_threshold = self.effective_threshold;
        let saved_level = self.level;

        self.function_like = true;
        self.effective_threshold = fn_threshold;
        self.level = 0;

        walk_block_mut(self, body, ctx);

        self.function_like = saved_function_like;
        self.effective_threshold = saved_threshold;
        self.level = saved_level;
    }
}

impl<'ctx, 'ast, 'arena> MutWalker<'ast, 'arena, LintContext<'ctx, 'arena>> for NestingWalker {
    fn walk_block(&mut self, block: &'ast Block<'arena>, ctx: &mut LintContext<'ctx, 'arena>) {
        self.level += 1;

        if !self.check_depth(block, ctx) {
            walk_block_mut(self, block, ctx);
        }

        self.level -= 1;
    }

    fn walk_in_function(&mut self, function: &'ast Function<'arena>, ctx: &mut LintContext<'ctx, 'arena>) {
        self.enter_function_like_body(&function.body, ctx);
    }

    fn walk_in_closure(&mut self, closure: &'ast Closure<'arena>, ctx: &mut LintContext<'ctx, 'arena>) {
        self.enter_function_like_body(&closure.body, ctx);
    }

    fn walk_in_method(&mut self, method: &'ast Method<'arena>, ctx: &mut LintContext<'ctx, 'arena>) {
        if let MethodBody::Concrete(body) = &method.body {
            self.enter_function_like_body(body, ctx);
        }
    }

    fn walk_in_property_hook_concrete_body(
        &mut self,
        body: &'ast PropertyHookConcreteBody<'arena>,
        ctx: &mut LintContext<'ctx, 'arena>,
    ) {
        if let PropertyHookConcreteBody::Block(block) = body {
            self.enter_function_like_body(block, ctx);
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::ExcessiveNestingRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = shallow_nesting,
        rule = ExcessiveNestingRule,
        code = indoc! {r#"
            <?php

            if ($a) {
                if ($b) {
                    echo "ok";
                }
            }
        "#}
    }

    test_lint_failure! {
        name = exceeds_default_threshold,
        rule = ExcessiveNestingRule,
        code = indoc! {r#"
            <?php

            if ($a) { if ($b) { if ($c) { if ($d) { if ($e) { if ($f) { if ($g) { if ($h) {
                echo "deep";
            } } } } } } } }
        "#}
    }

    test_lint_success! {
        name = within_custom_threshold,
        rule = ExcessiveNestingRule,
        settings = |s: &mut crate::settings::Settings| s.rules.excessive_nesting.config.threshold = 3,
        code = indoc! {r#"
            <?php

            if ($a) { if ($b) { if ($c) {
                echo "ok";
            } } }
        "#}
    }

    test_lint_failure! {
        name = exceeds_custom_threshold,
        rule = ExcessiveNestingRule,
        settings = |s: &mut crate::settings::Settings| s.rules.excessive_nesting.config.threshold = 2,
        code = indoc! {r#"
            <?php

            if ($a) { if ($b) { if ($c) {
                echo "deep";
            } } }
        "#}
    }

    test_lint_failure! {
        name = method_exceeds_function_like_threshold,
        rule = ExcessiveNestingRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.excessive_nesting.config.threshold = 100;
            s.rules.excessive_nesting.config.function_like_threshold = Some(2);
        },
        code = indoc! {r#"
            <?php

            class Foo {
                public function bar(): void {
                    if ($a) { if ($b) { if ($c) {
                        echo "deep in method";
                    } } }
                }
            }
        "#}
    }

    test_lint_success! {
        name = method_within_function_like_threshold,
        rule = ExcessiveNestingRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.excessive_nesting.config.threshold = 100;
            s.rules.excessive_nesting.config.function_like_threshold = Some(5);
        },
        code = indoc! {r#"
            <?php

            class Foo {
                public function bar(): void {
                    if ($a) { if ($b) { echo "ok"; } }
                }
            }
        "#}
    }

    test_lint_failure! {
        name = closure_exceeds_function_like_threshold,
        rule = ExcessiveNestingRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.excessive_nesting.config.threshold = 100;
            s.rules.excessive_nesting.config.function_like_threshold = Some(1);
        },
        code = indoc! {r#"
            <?php

            $fn = function () {
                if ($a) { if ($b) {
                    echo "deep";
                } }
            };
        "#}
    }

    test_lint_success! {
        name = no_function_like_threshold_preserves_bc,
        rule = ExcessiveNestingRule,
        settings = |s: &mut crate::settings::Settings| s.rules.excessive_nesting.config.threshold = 5,
        code = indoc! {r#"
            <?php

            class Foo {
                public function bar(): void {
                    if ($a) { if ($b) { echo "ok"; } }
                }
            }
        "#}
    }
}
