use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_casing::is_class_case;
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
pub struct InterfaceNameRule {
    meta: &'static RuleMeta,
    cfg: InterfaceNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct InterfaceNameConfig {
    pub level: Level,
    pub psr: bool,
}

impl Default for InterfaceNameConfig {
    fn default() -> Self {
        Self { level: Level::Help, psr: true }
    }
}

impl Config for InterfaceNameConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for InterfaceNameRule {
    type Config = InterfaceNameConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Interface Name",
            code: "interface-name",
            description: indoc! {"
                Detects interface declarations that do not follow class naming convention.

                Interface names should be in class case and suffixed with `Interface`, depending on the configuration.
            "},
            good_example: indoc! {r#"
                <?php

                interface MyInterface {}
            "#},
            bad_example: indoc! {r#"
                <?php

                interface myInterface {}
                interface my_interface {}
                interface MY_INTERFACE {}
            "#},
            category: Category::Consistency,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Interface];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Interface(interface) = node else {
            return;
        };

        let mut issues = vec![];
        let name = interface.name.value;
        let fqcn = ctx.lookup_name(&interface.name);

        if !is_class_case(name) {
            issues.push(
                Issue::new(self.cfg.level(), format!("Interface name `{}` should be in class case.", name))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(interface.name.span())
                            .with_message(format!("Interface `{}` is declared here", name)),
                    )
                    .with_annotation(
                        Annotation::secondary(interface.span())
                            .with_message(format!("Interface `{}` is defined here", fqcn)),
                    )
                    .with_note(format!("The interface name `{}` does not follow class naming convention.", name))
                    .with_help(format!(
                        "Consider renaming it to `{}` to adhere to the naming convention.",
                        mago_casing::to_class_case(name)
                    )),
            );
        }

        if self.cfg.psr && !name.ends_with("Interface") {
            issues.push(
                Issue::new(self.cfg.level(), format!("Interface name `{}` should be suffixed with `Interface`.", name))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(interface.name.span())
                            .with_message(format!("Interface `{}` is declared here", name)),
                    )
                    .with_annotation(
                        Annotation::secondary(interface.span())
                            .with_message(format!("Interface `{}` is defined here", fqcn)),
                    )
                    .with_note(format!("The interface name `{}` does not follow PSR naming convention.", name))
                    .with_help(format!(
                        "Consider renaming it to `{}Interface` to adhere to the naming convention.",
                        name
                    )),
            );
        }

        for issue in issues {
            ctx.collector.report(issue);
        }
    }
}
