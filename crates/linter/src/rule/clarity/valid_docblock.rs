use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct ValidDocblockRule {
    meta: &'static RuleMeta,
    cfg: ValidDocblockConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ValidDocblockConfig {
    pub level: Level,
}

impl Default for ValidDocblockConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for ValidDocblockConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ValidDocblockRule {
    type Config = ValidDocblockConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Valid Docblock",
            code: "valid-docblock",
            description: indoc! {"
                Checks for syntax errors in docblock comments. This rule is disabled by default because
                it can be noisy and may not be relevant to all codebases.
            "},
            good_example: indoc! {r#"
                <?php

                /**
                 * For more information, {@see https://example.com}.
                 *
                 * @param int $a
                 *
                 * @return int
                 */
                function foo($a) {
                    return $a;
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                /**
                 * For more information, {@see https://example.com
                 *
                 * @param int $a
                 *
                 * @return int
                 */
                function foo($a) {
                    return $a;
                }
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

        for trivia in program.trivia.iter() {
            if let TriviaKind::DocBlockComment = trivia.kind {
                let Err(parse_error) = mago_docblock::parse_trivia(ctx.arena, trivia) else {
                    continue;
                };

                let issue = Issue::new(self.cfg.level(), parse_error.to_string())
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(parse_error.span()))
                    .with_annotation(Annotation::secondary(trivia.span()))
                    .with_note(parse_error.note())
                    .with_help(parse_error.help());

                ctx.collector.report(issue);
            }
        }
    }
}
