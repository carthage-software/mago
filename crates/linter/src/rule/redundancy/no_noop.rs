use indoc::indoc;
use schemars::JsonSchema;

use mago_allocator::Arena;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::Statement;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::misc::STATEMENT_LIST_TARGETS;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoNoopRule {
    meta: &'static RuleMeta,
    cfg: NoNoopConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoNoopConfig {
    pub level: Level,
}

impl Default for NoNoopConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoNoopConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoNoopRule {
    type Config = NoNoopConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Noop",
            code: "no-noop",
            description: indoc! {"
                Detects redundant `noop` statements.
            "},
            good_example: indoc! {r#"
                <?php

                echo "Hello, world!";
            "#},
            bad_example: indoc! {r"
                <?php

                ;
            "},
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
            if let Statement::Noop(noop) = statement {
                let issue = Issue::new(self.cfg.level(), "Redundant noop statement.")
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(*noop).with_message("This is a redundant `noop` statement"))
                    .with_help("Remove the redundant `;`.");

                ctx.collector.propose(issue, |edits| {
                    edits.push(TextEdit::delete(*noop));
                });
            }
        }
    }
}
