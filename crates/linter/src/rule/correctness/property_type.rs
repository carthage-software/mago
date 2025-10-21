use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_reporting::*;
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
pub struct PropertyTypeRule {
    meta: &'static RuleMeta,
    cfg: PropertyTypeConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PropertyTypeConfig {
    pub level: Level,
}

impl Default for PropertyTypeConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for PropertyTypeConfig {
    fn default_enabled() -> bool {
        // This rule is disabled by default due to its deprecation.
        // Use the analyzer's `check-missing-type-hints` setting instead.
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PropertyTypeRule {
    type Config = PropertyTypeConfig;

    fn deprecated() -> bool {
        true
    }

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Property Type",
            code: "property-type",
            description: indoc! {"
                Detects class-like properties that are missing a type hint.
            "},
            good_example: indoc! {r#"
                <?php

                class Foo
                {
                    public int $bar;
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                class Foo
                {
                    public $bar;
                }
            "#},
            category: Category::Correctness,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP74)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Class, NodeKind::Trait];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let members = match node {
            Node::Class(class) => class.members.as_slice(),
            Node::Trait(r#trait) => r#trait.members.as_slice(),
            _ => return,
        };

        for member in members {
            let ClassLikeMember::Property(property) = member else {
                continue;
            };

            if property.hint().is_some() {
                continue;
            }

            for variable in property.variables() {
                ctx.collector.report(
                    Issue::new(self.cfg.level(), format!("Property `{}` is missing a type hint.", variable.name))
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(property.span())
                                .with_message(format!("Property `{}` is declared here.", variable.name)),
                        )
                        .with_note(
                            "Adding a type hint to properties improves code readability and helps prevent type errors.",
                        )
                        .with_help(format!("Consider specifying a type hint for `{}`.", variable.name)),
                );
            }
        }
    }
}
