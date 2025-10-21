use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::trivia::Trivia;
use mago_syntax::ast::trivia::TriviaKind;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoHashEmojiRule {
    meta: &'static RuleMeta,
    cfg: NoHashEmojiConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoHashEmojiConfig {
    pub level: Level,
}

impl Default for NoHashEmojiConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoHashEmojiConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoHashEmojiRule {
    type Config = NoHashEmojiConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Hash Emoji",
            code: "no-hash-emoji",
            description: indoc! {r#"
                Discourages usage of the `#️⃣` emoji in place of the ASCII `#`.

                While PHP allows the use of emojis in comments, it is generally discouraged to use them in place
                of the normal ASCII `#` symbol. This is because it can confuse readers and may break external
                tools that expect the normal ASCII `#` symbol.
            "#},
            good_example: indoc! {r#"
                <?php

                # This is a comment

                #[MyAttribute]
                class Foo {}
            "#},
            bad_example: indoc! {r#"
                <?php

                #️⃣ This is a comment

                #️⃣[MyAttribute] <- not a valid attribute
                class Foo {}
            "#},
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

        for trivia in &program.trivia.nodes {
            let Trivia { kind: TriviaKind::HashComment, value: comment, .. } = trivia else {
                continue;
            };

            if !comment.starts_with("#️⃣") {
                continue;
            }

            let mut issue = Issue::new(self.cfg.level, "Emoji-based hash (`#️⃣`) used instead of ASCII `#`.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(trivia.span()).with_message("This uses an emoji in place of `#`"))
                .with_note(
                    "While this might render similarly in some editors, it can confuse readers or break tooling.",
                )
                .with_help("Replace `#️⃣` with `#`.");

            if comment.starts_with("#️⃣[") {
                issue = issue.with_note("`#️⃣[` does not parse as an attribute in PHP; use `#[` instead.");
            }

            ctx.collector.propose(issue, |plan| {
                let trivia_span = trivia.span();
                let emoji_len = "#️⃣".len() as u32;

                plan.replace(
                    trivia_span.start.offset..(trivia_span.start.offset + emoji_len),
                    "#".to_string(),
                    SafetyClassification::Safe,
                );
            });
        }
    }
}
