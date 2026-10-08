use mago_allocator::Arena;
use schemars::JsonSchema;
use std::sync::LazyLock;

use indoc::indoc;
use regex::Regex;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::comments::comment_lines;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[allow(clippy::unwrap_used)]
static TAGGED_TODO_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"todo\((#|@)?\S+").unwrap());

#[derive(Debug, Clone)]
pub struct TaggedTodoRule {
    meta: &'static RuleMeta,
    cfg: TaggedTodoConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct TaggedTodoConfig {
    pub level: Level,
}

impl Default for TaggedTodoConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for TaggedTodoConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for TaggedTodoRule {
    type Config = TaggedTodoConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Tagged TODO",
            code: "tagged-todo",
            description: indoc! {"
                Detects TODO comments that are not tagged with a user or issue reference. Untagged TODOs
                can be difficult to track and may be forgotten. Tagging TODOs with a user or issue reference
                makes it easier to track progress and ensures that tasks are not forgotten.
            "},
            good_example: indoc! {r"
                <?php

                // TODO(@azjezz) This is a valid TODO comment.
                // TODO(azjezz) This is a valid TODO comment.
                // TODO(#123) This is a valid TODO comment.
            "},
            bad_example: indoc! {r"
                <?php

                // TODO: This is an invalid TODO comment.
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

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Program(program) = node else {
            return;
        };

        for trivia in &program.trivia {
            if !trivia.kind.is_comment() {
                continue;
            }

            for (_, line) in comment_lines(trivia) {
                let stripped =
                    line.iter().position(|&b| !b.is_ascii_whitespace()).map_or(&line[line.len()..], |i| &line[i..]);
                if !stripped.get(..4).is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"todo")) {
                    continue;
                }

                let trimmed_bytes = stripped.to_ascii_lowercase();
                let Some(trimmied) = std::str::from_utf8(&trimmed_bytes).ok() else {
                    continue;
                };

                if (*TAGGED_TODO_REGEX).is_match(trimmied) {
                    continue;
                }

                let issue = Issue::new(
                    self.cfg.level(),
                    "TODO should be tagged with (@username) or (#issue).",
                )
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(trivia.span).with_message("TODO comment is untagged"))
                .with_help(
                    "Add a user tag or issue reference to the TODO comment, e.g. TODO(@azjezz), TODO(azjezz), TODO(#123).",
                );

                ctx.collector.report(issue);

                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TaggedTodoRule;

    crate::test_lint_failure! {
        name = comment_prefixes_and_unicode_tags,
        rule = TaggedTodoRule,
        count = 3,
        code = "<?php\r\n// Plain comment\r\n// ToDo(@name)\r\n// todo note TODO(@other)\r\n// TODO(é)\r\n// to\r\n// \u{00a0}TODO\r\n// TODOfoo\r\n// TODO(\u{00a0} )\r\n/*\r\n * TODO:\r\n * TODO:\r\n */"
    }
}
