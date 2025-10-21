use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
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
pub struct NoShortOpeningTagRule {
    meta: &'static RuleMeta,
    cfg: NoShortOpeningTagConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoShortOpeningTagConfig {
    pub level: Level,
}

impl Default for NoShortOpeningTagConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoShortOpeningTagConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoShortOpeningTagRule {
    type Config = NoShortOpeningTagConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Short Opening Tag",
            code: "no-short-opening-tag",
            description: indoc! {r#"
                Disallows the use of short opening tags (`<?`).

                The availability of `<?` depends on the `short_open_tag` directive in `php.ini`. If
                this setting is disabled on a server, any code within the short tags will be
                exposed as plain text, which is a significant security risk. Using the full `<?php`
                opening tag is the only guaranteed portable way to ensure your code is always
                interpreted correctly.
            "#},
            good_example: indoc! {r#"
                <?php

                echo "Hello, World!";
            "#},
            bad_example: indoc! {r#"
                <?

                echo "Hello, World!";
            "#},
            category: Category::Security,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::ShortOpeningTag];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::ShortOpeningTag(opening_tag) = node else {
            return;
        };

        let issue = Issue::new(self.cfg.level, "Avoid using the short opening tag `<?`.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(opening_tag.span()).with_message("The short opening tag `<?` is used here"),
            )
            .with_note(
                "This tag's behavior depends on the `short_open_tag` setting in `php.ini`, which can be disabled.",
            )
            .with_note(
                "If disabled on the server, the enclosed PHP code will be exposed as plain text, creating a security vulnerability.",
            )
            .with_help("Always use the full `<?php` opening tag for portability and security.");

        ctx.collector.propose(issue, |plan| {
            plan.replace(opening_tag.span.to_range(), "<?php".to_string(), SafetyClassification::Safe);
        });
    }
}
