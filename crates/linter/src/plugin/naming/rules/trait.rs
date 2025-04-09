use indoc::indoc;
use toml::Value;

use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

const PSR: &str = "psr";
const PSR_DEFAULT: bool = true;

#[derive(Clone, Copy, Debug)]
pub struct TraitRule;

impl Rule for TraitRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Trait", Level::Help)
            .with_description(indoc! {"
                Detects trait declarations that do not follow class naming convention.
                Trait names should be in class case and suffixed with `Trait`, depending on the configuration.
            "})
            .with_option(RuleOptionDefinition {
                name: PSR,
                r#type: "boolean",
                description: "Enforce PSR naming convention, which requires trait names to be suffixed with `Trait`.",
                default: Value::Boolean(PSR_DEFAULT),
            })
            .with_example(RuleUsageExample::valid(
                "A trait name in class case",
                indoc! {r#"
                    <?php

                    trait MyTrait {}
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A trait name not in class case",
                indoc! {r#"
                    <?php

                    trait myTrait {}
                    trait my_trait {}
                    trait MY_TRAIT {}
                "#},
            ))
            .with_example(
                RuleUsageExample::valid(
                    "A trait not suffixed with `Trait`, with PSR option disabled",
                    indoc! {r#"
                    <?php

                    trait My {}
                "#},
                )
                .with_option(PSR, Value::Boolean(false)),
            )
            .with_example(
                RuleUsageExample::invalid(
                    "A trait not suffixed with `Trait`, with PSR option enabled",
                    indoc! {r#"
                    <?php

                    trait My {}
                "#},
                )
                .with_option(PSR, Value::Boolean(true)),
            )
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Trait(r#trait) = node else { return LintDirective::default() };

        let mut issues = vec![];

        let name = context.lookup(&r#trait.name.value);
        let fqcn = context.lookup_name(&r#trait.name);

        if !mago_casing::is_class_case(name) {
            issues.push(
                Issue::new(context.level(), format!("Trait name `{}` should be in class case.", name))
                    .with_annotations([
                        Annotation::primary(r#trait.name.span())
                            .with_message(format!("Trait `{}` is declared here.", name)),
                        Annotation::secondary(r#trait.span())
                            .with_message(format!("Trait `{}` is defined here.", fqcn)),
                    ])
                    .with_note(format!("The trait name `{}` does not follow class naming convention.", name))
                    .with_help(format!(
                        "Consider renaming it to `{}` to adhere to the naming convention.",
                        mago_casing::to_class_case(name)
                    )),
            );
        }

        if context.option(PSR).and_then(|o| o.as_bool()).unwrap_or(PSR_DEFAULT) && !name.ends_with("Trait") {
            issues.push(
                Issue::new(context.level(), format!("Trait name `{}` should be suffixed with `Trait`.", name))
                    .with_annotations([
                        Annotation::primary(r#trait.name.span())
                            .with_message(format!("Trait `{}` is declared here.", name)),
                        Annotation::secondary(r#trait.span())
                            .with_message(format!("Trait `{}` is defined here.", fqcn)),
                    ])
                    .with_note(format!("The trait name `{}` does not follow PSR naming convention.", name))
                    .with_help(format!("Consider renaming it to `{}Trait` to adhere to the naming convention.", name)),
            );
        }

        for issue in issues {
            context.report(issue);
        }

        LintDirective::default()
    }
}
