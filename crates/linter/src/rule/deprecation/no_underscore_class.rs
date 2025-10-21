use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
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
pub struct NoUnderscoreClassRule {
    meta: &'static RuleMeta,
    cfg: NoUnderscoreClassConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoUnderscoreClassConfig {
    pub level: Level,
}

impl Default for NoUnderscoreClassConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoUnderscoreClassConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoUnderscoreClassRule {
    type Config = NoUnderscoreClassConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Underscore Class",
            code: "no-underscore-class",
            description: indoc! {"
                Detects class, interface, trait, or enum declarations named `_`.

                Such names are considered deprecated; a more descriptive identifier is recommended.
            "},
            good_example: indoc! {r#"
                <?php

                class MyService {}
            "#},
            bad_example: indoc! {r#"
                <?php

                class _ {}
            "#},
            category: Category::Deprecation,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP84)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Class, NodeKind::Interface, NodeKind::Trait, NodeKind::Enum];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        match node {
            Node::Class(class) => {
                if class.name.value != "_" {
                    return;
                }

                ctx.collector.report(
                    Issue::new(self.cfg.level(), "Using `_` as a class name is deprecated.")
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(class.name.span())
                                .with_message("Rename the class to something more descriptive"),
                        )
                        .with_note(
                            "Class names consisting only of `_` are deprecated. Consider using a meaningful name.",
                        ),
                );
            }
            Node::Interface(interface) => {
                if interface.name.value != "_" {
                    return;
                }

                ctx.collector.report(
                    Issue::new(self.cfg.level(), "Using `_` as an interface name is deprecated.")
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(interface.name.span())
                                .with_message("Rename the interface to something more descriptive"),
                        )
                        .with_note(
                            "Interface names consisting only of `_` are deprecated. Consider using a meaningful name.",
                        ),
                );
            }
            Node::Trait(r#trait) => {
                if r#trait.name.value != "_" {
                    return;
                }

                ctx.collector.report(
                    Issue::new(self.cfg.level(), "Using `_` as a trait name is deprecated.")
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(r#trait.name.span())
                                .with_message("Rename the trait to something more descriptive"),
                        )
                        .with_note(
                            "Trait names consisting only of `_` are deprecated. Consider using a meaningful name.",
                        ),
                );
            }
            Node::Enum(r#enum) => {
                if r#enum.name.value != "_" {
                    return;
                }

                let issue = Issue::new(self.cfg.level(), "Using `_` as an enum name is deprecated.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(r#enum.name.span())
                            .with_message("Rename the enum to something more descriptive"),
                    )
                    .with_note("Enum names consisting only of `_` are deprecated. Consider using a meaningful name.");

                ctx.collector.report(issue);
            }
            _ => {}
        }
    }
}
