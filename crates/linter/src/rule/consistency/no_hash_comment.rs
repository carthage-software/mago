use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoHashCommentRule {
    meta: &'static RuleMeta,
    cfg: NoHashCommentConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoHashCommentConfig {
    pub level: Level,
}

impl Default for NoHashCommentConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoHashCommentConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoHashCommentRule {
    type Config = NoHashCommentConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Hash Comment",
            code: "no-hash-comment",
            description: indoc! {"
                Detects shell-style comments ('#') in PHP code. Double slash comments ('//') are preferred
                in PHP, as they are more consistent with the language's syntax and are easier to read.
            "},
            good_example: indoc! {"
                <?php

                // This is a good comment.
            "},
            bad_example: indoc! {"
                <?php

                # This is a shell-style comment.
            "},
            category: Category::Consistency,

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

        for trivia in program.trivia.iter() {
            if let TriviaKind::HashComment = trivia.kind {
                let issue = Issue::new(self.cfg.level(), "Shell-style comments ('#') are not allowed.")
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(trivia.span).with_message("This is a shell-style comment"))
                    .with_help("Consider using double slash comments ('//') instead.");

                ctx.collector.report(issue);
            }
        }
    }
}
