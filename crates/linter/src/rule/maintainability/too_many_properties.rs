use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::ClassLikeMember;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Property;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct TooManyPropertiesRule {
    meta: &'static RuleMeta,
    cfg: TooManyPropertiesConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct TooManyPropertiesConfig {
    pub level: Level,
    pub threshold: u16,
}

impl Default for TooManyPropertiesConfig {
    fn default() -> Self {
        Self { level: Level::Error, threshold: 10 }
    }
}

impl Config for TooManyPropertiesConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for TooManyPropertiesRule {
    type Config = TooManyPropertiesConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Too Many Properties",
            code: "too-many-properties",
            description: indoc::indoc! {r#"
                Detects class-like structures with too many properties.

                This rule checks the number of properties in classes, traits, and interfaces.
                If the number of properties exceeds a configurable threshold, an issue is reported.
            "#},
            good_example: indoc::indoc! {r#"
                <?php

                class SimpleClass {
                    public $a;
                    public $b;
                }
            "#},
            bad_example: indoc::indoc! {r#"
                <?php

                class ComplexClass {
                    public $a; public $b; public $c; public $d; public $e;
                    public $f; public $g; public $h; public $i; public $j; public $k;
                }
            "#},
            category: Category::Maintainability,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Class, NodeKind::Trait, NodeKind::Interface, NodeKind::AnonymousClass];

        TARGETS
    }

    fn build(settings: &RuleSettings<TooManyPropertiesConfig>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let members = match node {
            Node::Class(c) => c.members.as_slice(),
            Node::Trait(t) => t.members.as_slice(),
            Node::Interface(i) => i.members.as_slice(),
            Node::AnonymousClass(i) => i.members.as_slice(),
            _ => return,
        };

        let mut properties = 0;
        for member in members {
            let ClassLikeMember::Property(property) = member else {
                continue;
            };

            match property {
                Property::Plain(plain_property) => {
                    properties += plain_property.items.len();
                }
                Property::Hooked(_) => {
                    properties += 1;
                }
            }
        }

        if properties > self.cfg.threshold as usize {
            let kind = match node.kind() {
                NodeKind::Class => "Class",
                NodeKind::Trait => "Trait",
                NodeKind::Interface => "Interface",
                NodeKind::AnonymousClass => "Anonymous class",
                _ => unreachable!(),
            };

            ctx.collector.report(
                Issue::new(self.cfg.level, format!("{kind} has too many properties."))
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(node.span()).with_message(format!(
                        "{kind} has {properties} properties, which exceeds the threshold of {}.",
                        self.cfg.threshold
                    )))
                    .with_note("Having a large number of properties can make classes harder to understand and maintain.")
                    .with_help("Try reducing the number of properties, or consider grouping related properties into a single object.")
            );

            // If this structure has too many props, we don't need to check the nested structures.
        } else {
            // Continue checking nested structures, if any.
        }
    }
}
