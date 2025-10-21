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
pub struct NoPhpTagTerminatorRule {
    meta: &'static RuleMeta,
    cfg: NoPhpTagTerminatorConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoPhpTagTerminatorConfig {
    pub level: Level,
}

impl Default for NoPhpTagTerminatorConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for NoPhpTagTerminatorConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoPhpTagTerminatorRule {
    type Config = NoPhpTagTerminatorConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Php Tag Terminator",
            code: "no-php-tag-terminator",
            description: indoc! {"
                Discourages the use of `?><?php` as a statement terminator. Recommends using a semicolon
                (`;`) instead for clarity and consistency.
            "},
            good_example: indoc! {r#"
                <?php

                echo "Hello World";
            "#},
            bad_example: indoc! {r#"
                <?php

                echo "Hello World" ?><?php
            "#},
            category: Category::Consistency,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Terminator];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Terminator(terminator) = node else {
            return;
        };
        let Terminator::TagPair(close, open) = terminator else {
            return;
        };

        let issue = Issue::new(self.cfg.level(), "Semicolon terminator is preferred over tag-pair terminator.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(close.span().join(open.span()))
                    .with_message("This tag-pair terminator `?><?php` is not recommended"),
            )
            .with_help("Replace `?><?php` with a `;`.");

        ctx.collector.report(issue);
    }
}
