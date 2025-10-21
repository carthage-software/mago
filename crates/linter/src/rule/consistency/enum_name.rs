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
pub struct EnumNameRule {
    meta: &'static RuleMeta,
    cfg: EnumNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct EnumNameConfig {
    pub level: Level,
}

impl Default for EnumNameConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for EnumNameConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for EnumNameRule {
    type Config = EnumNameConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Enum Name",
            code: "enum-name",
            description: indoc! {"
                Detects enum declarations that do not follow class naming convention.

                Enum names should be in class case, also known as PascalCase.
            "},
            good_example: indoc! {r#"
                <?php

                enum MyEnum {}
            "#},
            bad_example: indoc! {r#"
                <?php

                enum my_enum {}
                enum myEnum {}
                enum MY_ENUM {}
            "#},
            category: Category::Consistency,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Enum];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Enum(r#enum) = node else {
            return;
        };

        let name = r#enum.name.value;
        let fqcn = ctx.lookup_name(&r#enum.name);

        if !is_class_case(name) {
            ctx.collector.report(
                Issue::new(self.cfg.level(), format!("Enum name `{}` should be in class case.", name))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(r#enum.name.span())
                            .with_message(format!("Enum `{}` is declared here", name)),
                    )
                    .with_annotation(
                        Annotation::secondary(r#enum.span()).with_message(format!("Enum `{}` is defined here", fqcn)),
                    )
                    .with_note(format!("The enum name `{}` does not follow class naming convention.", name))
                    .with_help(format!(
                        "Consider renaming it to `{}` to adhere to the naming convention.",
                        to_class_case(name)
                    )),
            );
        }

        if r#enum.members.contains_methods() {}
    }
}
