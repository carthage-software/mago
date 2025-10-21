use std::sync::LazyLock;

use indoc::indoc;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::ast::*;
use mago_syntax::comments::comment_lines;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

static TAGGED_FIXME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"fixme\((#|@)?\S+").unwrap());

#[derive(Debug, Clone)]
pub struct TaggedFixmeRule {
    meta: &'static RuleMeta,
    cfg: TaggedFixmeConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct TaggedFixmeConfig {
    pub level: Level,
}

impl Default for TaggedFixmeConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for TaggedFixmeConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for TaggedFixmeRule {
    type Config = TaggedFixmeConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Tagged FIXME",
            code: "tagged-fixme",
            description: indoc! {"
                Detects FIXME comments that are not tagged with a user or issue reference. Untagged FIXME comments
                are not actionable and can be easily missed by the team. Tagging the FIXME comment with a user or
                issue reference ensures that the issue is tracked and resolved.
            "},
            good_example: indoc! {"
                <?php

                // FIXME(@azjezz) This is a valid FIXME comment.
                // FIXME(azjezz) This is a valid FIXME comment.
                // FIXME(#123) This is a valid FIXME comment.
            "},
            bad_example: indoc! {"
                <?php

                // FIXME: This is an invalid FIXME comment.
            "},
            category: Category::Clarity,

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
            if !trivia.kind.is_comment() {
                continue;
            }

            for (_, line) in comment_lines(trivia) {
                let trimmied = line.trim_start().to_lowercase();
                if !trimmied.starts_with("fixme") {
                    continue;
                }

                if (*TAGGED_FIXME_REGEX).is_match(&trimmied) {
                    continue;
                }

                let issue = Issue::new(
                    self.cfg.level(),
                    "FIXME comment should be tagged with (@username) or (#issue).",
                )
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(trivia.span))
                .with_help(
                    "Add a user tag or issue reference to the FIXME comment, e.g. FIXME(@azjezz), FIXME(azjezz), FIXME(#123).",
                );

                ctx.collector.report(issue);

                break;
            }
        }
    }
}
