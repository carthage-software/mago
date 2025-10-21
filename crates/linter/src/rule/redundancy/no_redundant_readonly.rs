use indoc::indoc;
use mago_fixer::SafetyClassification;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::ClassLikeMember;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoRedundantReadonlyRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantReadonlyConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantReadonlyConfig {
    pub level: Level,
}

impl Default for NoRedundantReadonlyConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoRedundantReadonlyConfig {
    fn default_enabled() -> bool {
        // TODO(azjezz): enable in the next major release
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantReadonlyRule {
    type Config = NoRedundantReadonlyConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Readonly",
            code: "no-redundant-readonly",
            description: indoc! {"
                Detects redundant readonly modifiers on properties.
            "},
            good_example: indoc! {r#"
                <?php

                readonly class User
                {
                    public $name;
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                readonly class User
                {
                    public readonly $name;
                }
            "#},
            category: Category::Redundancy,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Class];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Class(class) = node else {
            return;
        };

        if !class.modifiers.contains_readonly() {
            return;
        }

        for member in class.members.iter() {
            if let ClassLikeMember::Property(property) = member
                && let Some(readonly_modifier) = property.modifiers().get_readonly()
            {
                let issue = Issue::new(
                    self.cfg.level(),
                    "The `readonly` modifier is redundant as the class is already readonly.",
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(readonly_modifier.span()).with_message("This `readonly` modifier is redundant"),
                )
                .with_help("Remove the redundant `readonly` modifier.");

                ctx.collector.propose(issue, |plan| {
                    plan.delete(readonly_modifier.span().to_range(), SafetyClassification::Safe);
                });
            }
        }
    }
}
