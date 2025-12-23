use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Statement;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct RequireNamespaceRule {
    meta: &'static RuleMeta,
    cfg: RequireNamespaceConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct RequireNamespaceConfig {
    pub level: Level,
}

impl Default for RequireNamespaceConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for RequireNamespaceConfig {
    fn level(&self) -> Level {
        self.level
    }

    fn default_enabled() -> bool {
        false
    }
}

impl LintRule for RequireNamespaceRule {
    type Config = RequireNamespaceConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Require Namespace",
            code: "require-namespace",
            description: indoc! {"
                Detects files that contain definitions (classes, interfaces, enums, traits, functions, or constants)
                but do not declare a namespace. Using namespaces helps avoid naming conflicts and improves code organization.
            "},
            good_example: indoc! {r#"
                <?php

                namespace App;

                class Foo {}
            "#},
            bad_example: indoc! {r#"
                <?php

                class Foo {}
            "#},
            category: Category::BestPractices,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Program];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        let mut has_namespace = false;
        let mut first_definition_span = None;

        for statement in &program.statements {
            match statement {
                Statement::Namespace(_) => {
                    has_namespace = true;
                    break;
                }
                Statement::Class(class) => {
                    if first_definition_span.is_none() {
                        first_definition_span = Some(class.span());
                    }
                }
                Statement::Interface(interface) => {
                    if first_definition_span.is_none() {
                        first_definition_span = Some(interface.span());
                    }
                }
                Statement::Enum(e) => {
                    if first_definition_span.is_none() {
                        first_definition_span = Some(e.span());
                    }
                }
                Statement::Trait(t) => {
                    if first_definition_span.is_none() {
                        first_definition_span = Some(t.span());
                    }
                }
                Statement::Function(f) => {
                    if first_definition_span.is_none() {
                        first_definition_span = Some(f.span());
                    }
                }
                Statement::Constant(c) => {
                    if first_definition_span.is_none() {
                        first_definition_span = Some(c.span());
                    }
                }
                _ => {}
            }
        }

        if has_namespace {
            return;
        }

        let Some(definition_span) = first_definition_span else {
            return;
        };

        let issue = Issue::new(self.cfg.level(), "File contains definitions without a namespace declaration.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(definition_span).with_message("Definition found without namespace"))
            .with_note("Using namespaces helps avoid naming conflicts and improves code organization.")
            .with_help("Add a namespace declaration at the top of the file.");

        ctx.collector.report(issue);
    }
}
