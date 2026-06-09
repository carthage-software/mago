use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_casing::is_class_case;
use mago_casing::to_class_case;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::ClassLikeMemberSequenceExt;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;
use mago_bytes::BytesDisplay;

#[derive(Debug, Clone)]
pub struct EnumNameRule {
    meta: &'static RuleMeta,
    cfg: EnumNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
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
            good_example: indoc! {r"
                <?php

                enum MyEnum {}
            "},
            bad_example: indoc! {r"
                <?php

                enum my_enum {}
                enum myEnum {}
                enum MY_ENUM {}
            "},
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

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Enum(r#enum) = node else {
            return;
        };

        let Some(name) = std::str::from_utf8(r#enum.name.value).ok() else { return };
        let fqcn = BytesDisplay(ctx.lookup_name(&r#enum.name));

        if !is_class_case(name) {
            ctx.collector.report(
                Issue::new(self.cfg.level(), format!("Enum name `{name}` should be in class case."))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(r#enum.name.span()).with_message(format!("Enum `{name}` is declared here")),
                    )
                    .with_annotation(
                        Annotation::secondary(r#enum.span()).with_message(format!("Enum `{fqcn}` is defined here")),
                    )
                    .with_note(format!("The enum name `{name}` does not follow class naming convention."))
                    .with_help(format!(
                        "Consider renaming it to `{}` to adhere to the naming convention.",
                        String::from_utf8_lossy(&to_class_case(name))
                    )),
            );
        }

        if r#enum.members.contains_methods() {}
    }
}
