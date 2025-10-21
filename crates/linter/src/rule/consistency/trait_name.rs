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
pub struct TraitNameRule {
    meta: &'static RuleMeta,
    cfg: TraitNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct TraitNameConfig {
    pub level: Level,
    pub psr: bool,
}

impl Default for TraitNameConfig {
    fn default() -> Self {
        Self { level: Level::Help, psr: true }
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
            good_example: indoc! {r#"
                <?php

                trait MyTrait {}
            "#},
            bad_example: indoc! {r#"
                <?php

                trait myTrait {}
                trait my_trait {}
                trait MY_TRAIT {}
            "#},
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

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Trait(r#trait) = node else {
            return;
        };

        let mut issues = vec![];

        let name = r#trait.name.value;
        let fqcn = ctx.lookup_name(&r#trait.name);

        if !is_class_case(name) {
            issues.push(
                Issue::new(self.cfg.level(), format!("Trait name `{}` should be in class case.", name))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(r#trait.name.span())
                            .with_message(format!("Trait `{}` is declared here", name)),
                    )
                    .with_annotation(
                        Annotation::secondary(r#trait.span()).with_message(format!("Trait `{}` is defined here", fqcn)),
                    )
                    .with_note(format!("The trait name `{}` does not follow class naming convention.", name))
                    .with_help(format!(
                        "Consider renaming it to `{}` to adhere to the naming convention.",
                        mago_casing::to_class_case(name)
                    )),
            );
        }

        if self.cfg.psr && !name.ends_with("Trait") {
            issues.push(
                Issue::new(self.cfg.level(), format!("Trait name `{}` should be suffixed with `Trait`.", name))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(r#trait.name.span())
                            .with_message(format!("Trait `{}` is declared here", name)),
                    )
                    .with_annotation(
                        Annotation::secondary(r#trait.span()).with_message(format!("Trait `{}` is defined here", fqcn)),
                    )
                    .with_note(format!("The trait name `{}` does not follow PSR naming convention.", name))
                    .with_help(format!("Consider renaming it to `{}Trait` to adhere to the naming convention.", name)),
            );
        }

        for issue in issues {
            ctx.collector.report(issue);
        }
    }
}
