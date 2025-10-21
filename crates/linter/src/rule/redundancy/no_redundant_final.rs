use indoc::indoc;
use mago_fixer::SafetyClassification;
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
pub struct NoRedundantFinalRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantFinalConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantFinalConfig {
    pub level: Level,
}

impl Default for NoRedundantFinalConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoRedundantFinalConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantFinalRule {
    type Config = NoRedundantFinalConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Final",
            code: "no-redundant-final",
            description: indoc! {"
                Detects redundant `final` modifiers on methods in final classes or enum methods.
            "},
            good_example: indoc! {r#"
                <?php

                final class Foo {
                    public function bar(): void {
                        // ...
                    }
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                final class Foo {
                    final public function bar(): void {
                        // ...
                    }
                }
            "#},
            category: Category::Redundancy,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Class, NodeKind::Enum];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let (members, is_enum) = match node {
            Node::Class(class) => {
                if !class.modifiers.contains_final() {
                    return;
                }

                (&class.members, false)
            }
            Node::Enum(r#enum) => (&r#enum.members, true),
            _ => return,
        };

        if !members.contains_methods() {
            return;
        }

        for member in members.iter() {
            if let ClassLikeMember::Method(method) = member {
                let Some(final_modifier) = method.modifiers.get_final() else {
                    continue;
                };

                let message = if is_enum {
                    format!(
                        "The `final` modifier on enum method `{}` is redundant as enums cannot be extended.",
                        method.name.value,
                    )
                } else {
                    format!(
                        "The `final` modifier on method `{}` is redundant as the class is already final.",
                        method.name.value,
                    )
                };

                let issue = Issue::new(self.cfg.level(), message)
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(final_modifier.span()).with_message("This `final` modifier is redundant"),
                    )
                    .with_help("Remove the redundant `final` modifier.");

                ctx.collector.propose(issue, |plan| {
                    plan.delete(final_modifier.span().to_range(), SafetyClassification::Safe);
                });
            }
        }
    }
}
