use indoc::indoc;
use schemars::JsonSchema;

use mago_allocator::Arena;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::Statement;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::misc::STATEMENT_LIST_TARGETS;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoRedundantBlockRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantBlockConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoRedundantBlockConfig {
    pub level: Level,
}

impl Default for NoRedundantBlockConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoRedundantBlockConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantBlockRule {
    type Config = NoRedundantBlockConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Block",
            code: "no-redundant-block",
            description: indoc! {"
                Detects redundant blocks around statements.
            "},
            good_example: indoc! {r#"
                <?php

                echo "Hello, world!";
            "#},
            bad_example: indoc! {r#"
                <?php

                {
                    echo "Hello, world!";
                }
            "#},
            category: Category::Redundancy,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        STATEMENT_LIST_TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Some(statements) = crate::rule::utils::misc::statement_list_of(node) else {
            return;
        };

        for statement in statements {
            if let Statement::Block(block) = statement {
                let issue = Issue::new(self.cfg.level(), "Redundant block around statements.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(block.span())
                            .with_message("Statements do not need to be wrapped within a block"),
                    )
                    .with_help("Remove the block to simplify the code.");

                ctx.collector.report(issue);
            }
        }
    }
}
