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
use crate::rule::utils::misc::is_method_setter_or_getter;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct TooManyMethodsRule {
    meta: &'static RuleMeta,
    cfg: TooManyMethodsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct TooManyMethodsConfig {
    pub level: Level,
    pub threshold: u16,
    pub count_hooks: bool,
    pub count_setters_and_getters: bool,
}

impl Default for TooManyMethodsConfig {
    fn default() -> Self {
        Self { level: Level::Error, threshold: 10, count_hooks: false, count_setters_and_getters: false }
    }
}

impl Config for TooManyMethodsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for TooManyMethodsRule {
    type Config = TooManyMethodsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Too Many Methods",
            code: "too-many-methods",
            description: indoc::indoc! {r#"
                Detects class-like structures with too many methods.

                This rule checks the number of methods in classes, traits, enums, and interfaces.
                If the number of methods exceeds a configurable threshold, an issue is reported.
            "#},
            good_example: indoc::indoc! {r#"
                <?php

                class SimpleClass {
                    public function a() {}
                    public function b() {}
                }
            "#},
            bad_example: indoc::indoc! {r#"
                <?php

                class ComplexClass {
                    public function a() {}
                    public function b() {}
                    public function c() {}
                    public function d() {}
                    public function e() {}
                    public function f() {}
                    public function g() {}
                    public function h() {}
                    public function i() {}
                    public function j() {}
                    public function k() {}
                    public function l() {}
                    public function m() {}
                    public function n() {}
                    public function o() {}
                    public function p() {}
                    public function q() {}
                    public function r() {}
                    public function s() {}
                    public function t() {}
                    public function u() {}
                }
            "#},
            category: Category::Maintainability,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] =
            &[NodeKind::Class, NodeKind::Trait, NodeKind::Enum, NodeKind::Interface, NodeKind::AnonymousClass];

        TARGETS
    }

    fn build(settings: &RuleSettings<TooManyMethodsConfig>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let members = match node {
            Node::Class(class) => class.members.as_slice(),
            Node::Trait(r#trait) => r#trait.members.as_slice(),
            Node::Enum(r#enum) => r#enum.members.as_slice(),
            Node::Interface(interface) => interface.members.as_slice(),
            Node::AnonymousClass(class) => class.members.as_slice(),
            _ => return,
        };

        let mut count = 0;
        for member in members {
            match member {
                ClassLikeMember::Method(method) => {
                    if !self.cfg.count_setters_and_getters && is_method_setter_or_getter(method) {
                        continue;
                    }

                    count += 1;
                }
                ClassLikeMember::Property(Property::Hooked(hooked_property)) if self.cfg.count_hooks => {
                    count += hooked_property.hook_list.hooks.len();
                }
                _ => (),
            }
        }

        if count as u16 > self.cfg.threshold {
            let kind = match node.kind() {
                NodeKind::Class => "Class",
                NodeKind::Trait => "Trait",
                NodeKind::Enum => "Enum",
                NodeKind::Interface => "Interface",
                NodeKind::AnonymousClass => "Anonymous class",
                _ => unreachable!(),
            };

            ctx.collector.report(
                Issue::new(self.cfg.level, format!("{kind} has too many methods."))
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(node.span()).with_message(format!(
                        "{kind} has {count} methods, which exceeds the threshold of {}.",
                        self.cfg.threshold
                    )))
                    .with_note("Having a large number of methods can make structures harder to understand and maintain.")
                    .with_help("Try reducing the number of methods, or consider splitting the structure into smaller, more focused structures.")
            );

            // If this structure has too many methods, we don't need to check the nested structures.
        } else {
            // Continue checking nested structures, if any.
        }
    }
}
