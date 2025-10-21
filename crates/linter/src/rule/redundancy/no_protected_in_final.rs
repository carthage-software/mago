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
pub struct NoProtectedInFinalRule {
    meta: &'static RuleMeta,
    cfg: NoProtectedInFinalConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoProtectedInFinalConfig {
    pub level: Level,
}

impl Default for NoProtectedInFinalConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoProtectedInFinalConfig {
    fn default_enabled() -> bool {
        // TODO(azjezz): enable in the next major release
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoProtectedInFinalRule {
    type Config = NoProtectedInFinalConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Protected in Final",
            code: "no-protected-in-final",
            description: indoc! {"
                Detects `protected` items in final classes or enums.
            "},
            good_example: indoc! {r#"
                <?php

                final class Foo {
                    private string $foo;
                    private(set) string $bar;
                    private string $baz;

                    private function fun(): void {
                        // ...
                    }
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                final class Foo {
                    protected string $foo;
                    protected(set) string $bar;
                    protected private(set) string $baz;

                    protected function fun(): void {
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
        let (members, message) = match node {
            Node::Class(class) if class.modifiers.contains_final() => {
                (&class.members, "The `protected` visibility can be made `private` as the class is final.")
            }
            Node::Enum(r#enum) => {
                (&r#enum.members, "The `protected` visibility can be made `private` as enums cannot be extended.")
            }
            _ => return,
        };

        for member in members.iter() {
            let modifiers = match member {
                ClassLikeMember::Method(method) => &method.modifiers,
                ClassLikeMember::Property(property) => property.modifiers(),
                _ => continue,
            };

            if let Some(protected_modifier) = modifiers.get_protected().or_else(|| modifiers.get_protected_set()) {
                let span = protected_modifier.span();

                let issue = Issue::new(self.cfg.level(), message)
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(span).with_message("This `protected` visibility can be made `private`"),
                    )
                    .with_help("Replace the `protected` visibility by `private`.");

                if protected_modifier.is_protected_set() {
                    ctx.collector.propose(issue, |plan| {
                        plan.replace(span.to_range(), "private(set)", SafetyClassification::Safe);
                    });
                } else if let Some(set_modifier) = modifiers.get_first_write_visibility() {
                    let issue = issue.with_annotation(
                        Annotation::secondary(set_modifier.span())
                            .with_message("This write visibility will become redundant"),
                    );
                    ctx.collector.propose(issue, |plan| {
                        plan.replace(span.to_range(), "private", SafetyClassification::Safe);
                        plan.delete(set_modifier.span().to_range(), SafetyClassification::Safe);
                    });
                } else {
                    ctx.collector.propose(issue, |plan| {
                        plan.replace(span.to_range(), "private", SafetyClassification::Safe);
                    });
                }
            }
        }
    }
}
