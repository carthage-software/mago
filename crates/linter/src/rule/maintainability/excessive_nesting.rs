use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Block;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ExcessiveNestingConfig {
    pub level: Level,
    pub threshold: usize,
}

impl Default for ExcessiveNestingConfig {
    fn default() -> Self {
        Self { level: Level::Warning, threshold: DEFAULT_THRESHOLD }
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
            description: indoc! {r#"
                Checks if the nesting level in any block exceeds a configurable threshold.

                Deeply nested code is harder to read, understand, and maintain.
                Consider refactoring to use early returns, helper methods, or clearer control flow.
            "#},
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

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        let mut walker = NestingWalker { threshold: self.cfg.threshold, level: 0, meta: self.meta, cfg: self.cfg };

        walker.walk_program(program, ctx);
    }
}

struct NestingWalker {
    threshold: usize,
    level: usize,
    meta: &'static RuleMeta,
    cfg: ExcessiveNestingConfig,
}

impl NestingWalker {
    fn check_depth(&self, block: &Block, ctx: &mut LintContext) -> bool {
        if self.level > self.threshold {
            let issue = Issue::new(
                self.cfg.level,
                "Excessive block nesting.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(block.span())
                    .with_message(format!(
                        "This block has a nesting depth of {lvl}, which exceeds the configured threshold of {thr}.",
                        lvl = self.level,
                        thr = self.threshold
                    )),
            )
            .with_note(format!(
                "The current nesting level is {lvl}, which is greater than the allowed threshold of {thr}.",
                lvl = self.level,
                thr = self.threshold
            ))
            .with_note("Excessive nesting can make code harder to read, understand, and maintain.")
            .with_help("Refactor your code to reduce nesting (e.g. use early returns, guard clauses, or helper functions).");

            ctx.collector.report(issue);
            return true;
        }

        false
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
}
