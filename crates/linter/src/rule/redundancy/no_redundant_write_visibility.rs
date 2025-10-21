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
pub struct NoRedundantWriteVisibilityRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantWriteVisibilityConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantWriteVisibilityConfig {
    pub level: Level,
}

impl Default for NoRedundantWriteVisibilityConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoRedundantWriteVisibilityConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantWriteVisibilityRule {
    type Config = NoRedundantWriteVisibilityConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Write Visibility",
            code: "no-redundant-write-visibility",
            description: indoc! {"
                Detects redundant write visibility modifiers on properties.
                "},
            good_example: indoc! {r#"
                <?php

                final class User
                {
                    public $name;
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                final class User
                {
                    public public(set) $name;
                }
            "#},
            category: Category::Redundancy,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Property];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Property(property) = node else {
            return;
        };

        let modifiers = property.modifiers();
        if modifiers.is_empty() {
            return;
        }

        let Some(write_visibility) = modifiers.get_first_write_visibility() else {
            return;
        };

        let Some(read_visibility) = modifiers.get_first_read_visibility() else {
            return;
        };

        match (read_visibility, write_visibility) {
            (Modifier::Public(_), Modifier::PublicSet(_))
            | (Modifier::Protected(_), Modifier::ProtectedSet(_))
            | (Modifier::Private(_), Modifier::PrivateSet(_)) => {
                let issue = Issue::new(self.cfg.level(), "Identical write visibility has no effect.")
                    .with_code(self.meta.code)
                    .with_help("Remove the redundant write visibility modifier.")
                    .with_annotations(vec![
                        Annotation::primary(write_visibility.span()).with_message("Redundant write visibility"),
                        Annotation::secondary(read_visibility.span()).with_message("Read visibility"),
                    ]);

                ctx.collector.report(issue);
            }
            _ => {}
        }
    }
}
