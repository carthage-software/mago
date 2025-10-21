use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_casing::is_class_case;
use mago_casing::to_class_case;
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
pub struct ClassNameRule {
    meta: &'static RuleMeta,
    cfg: ClassNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ClassNameConfig {
    pub level: Level,
    pub psr: bool,
}

impl Default for ClassNameConfig {
    fn default() -> Self {
        Self { level: Level::Help, psr: true }
    }
}

impl Config for ClassNameConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ClassNameRule {
    type Config = ClassNameConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Class Name",
            code: "class-name",
            description: indoc! {"
                Detects class declarations that do not follow class naming convention.

                Class names should be in class case, also known as PascalCase.
            "},
            good_example: indoc! {r#"
                <?php

                class MyClass {}
            "#},
            bad_example: indoc! {r#"
                <?php

                class my_class {}

                class myClass {}

                class MY_CLASS {}
            "#},
            category: Category::Consistency,

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
        let Node::Class(class) = node else { return };
        let mut issues = vec![];
        let name = class.name.value;

        if !is_class_case(name) {
            let issue = Issue::new(self.cfg.level(), format!("Class name `{}` should be in class case.", name))
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(class.name.span()).with_message(format!("Class `{}` is declared here", name)),
                )
                .with_note(format!("The class name `{}` does not follow class naming convention.", name))
                .with_help(format!(
                    "Consider renaming it to `{}` to adhere to the naming convention.",
                    to_class_case(name)
                ));

            issues.push(issue);
        }

        if class.modifiers.contains_abstract() && self.cfg.psr && !name.starts_with("Abstract") {
            let suggested_name = format!("Abstract{}", to_class_case(name));

            issues.push(
                Issue::new(
                    self.cfg.level(),
                    format!("Abstract class name `{}` should be prefixed with `Abstract`.", name),
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(class.name.span()).with_message(format!("Class `{}` is declared here", name)),
                )
                .with_note(format!("The abstract class name `{}` does not follow PSR naming convention.", name))
                .with_help(format!("Consider renaming it to `{}` to adhere to the naming convention.", suggested_name)),
            );
        }

        for issue in issues {
            ctx.collector.report(issue);
        }
    }
}
