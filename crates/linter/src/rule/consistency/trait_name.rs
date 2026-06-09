use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_casing::is_class_case;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
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
pub struct TraitNameRule {
    meta: &'static RuleMeta,
    cfg: TraitNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct TraitNameConfig {
    pub level: Level,
    pub psr: bool,
}

impl Default for TraitNameConfig {
    fn default() -> Self {
        Self { level: Level::Help, psr: false }
    }
}

impl Config for TraitNameConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for TraitNameRule {
    type Config = TraitNameConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Trait Name",
            code: "trait-name",
            description: indoc! {"
                Detects trait declarations that do not follow class naming convention.
                Trait names should be in class case and suffixed with `Trait`, depending on the configuration.
            "},
            good_example: indoc! {r"
                <?php

                trait MyTrait {}
            "},
            bad_example: indoc! {r"
                <?php

                trait myTrait {}
                trait my_trait {}
                trait MY_TRAIT {}
            "},
            category: Category::Consistency,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Trait];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Trait(r#trait) = node else {
            return;
        };

        let mut issues = vec![];

        let name_bytes = r#trait.name.value;
        let Some(name) = std::str::from_utf8(name_bytes).ok() else { return };
        let fqcn = BytesDisplay(ctx.lookup_name(&r#trait.name));

        if !is_class_case(name) {
            issues.push(
                Issue::new(self.cfg.level(), format!("Trait name `{name}` should be in class case."))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(r#trait.name.span())
                            .with_message(format!("Trait `{name}` is declared here")),
                    )
                    .with_annotation(
                        Annotation::secondary(r#trait.span()).with_message(format!("Trait `{fqcn}` is defined here")),
                    )
                    .with_note(format!("The trait name `{name}` does not follow class naming convention."))
                    .with_help(format!(
                        "Consider renaming it to `{}` to adhere to the naming convention.",
                        String::from_utf8_lossy(&mago_casing::to_class_case(name))
                    )),
            );
        }

        if self.cfg.psr && !name_bytes.ends_with(b"Trait") {
            issues.push(
                Issue::new(self.cfg.level(), format!("Trait name `{name}` should be suffixed with `Trait`."))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(r#trait.name.span())
                            .with_message(format!("Trait `{name}` is declared here")),
                    )
                    .with_annotation(
                        Annotation::secondary(r#trait.span()).with_message(format!("Trait `{fqcn}` is defined here")),
                    )
                    .with_note(format!("The trait name `{name}` does not follow PSR naming convention."))
                    .with_help(format!("Consider renaming it to `{name}Trait` to adhere to the naming convention.")),
            );
        }

        for issue in issues {
            ctx.collector.report(issue);
        }
    }
}
