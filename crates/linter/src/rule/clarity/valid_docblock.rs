use indoc::indoc;
use mago_allocator::Arena;
use mago_phpdoc_syntax::PHPDocParser;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::TriviaKind;

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
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
                Checks for syntax errors in docblock comments, such as malformed `{@see}` or
                `{@link}` annotations. It does not enforce the presence of docblocks or verify
                that declared types match the native declaration.
            "},
            good_example: indoc! {r"
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
            "},
            bad_example: indoc! {r"
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
            if trivia.kind != TriviaKind::DocBlockComment {
                continue;
            }

            let document = PHPDocParser::parse_with_span(ctx.arena, trivia.value, trivia.span);
            for parse_error in document.errors {
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
