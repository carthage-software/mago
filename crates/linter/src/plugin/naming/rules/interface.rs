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
pub struct InterfaceRule;

impl Rule for InterfaceRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Interface", Level::Help)
            .with_description(indoc! {"
                Detects interface declarations that do not follow class naming convention.
                Interface names should be in class case and suffixed with `Interface`, depending on the configuration.
            "})
            .with_option(RuleOptionDefinition {
                name: PSR,
                r#type: "boolean",
                description:
                    "Enforce PSR naming convention, which requires interface names to be suffixed with `Interface`.",
                default: Value::Boolean(PSR_DEFAULT),
            })
            .with_example(RuleUsageExample::valid(
                "An interface name in class case",
                indoc! {r#"
                    <?php

                    interface MyInterface {}
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "An interface name not in class case",
                indoc! {r#"
                    <?php

                    interface myInterface {}
                    interface my_interface {}
                    interface MY_INTERFACE {}
                "#},
            ))
            .with_example(
                RuleUsageExample::valid(
                    "An interface not suffixed with `Interface`, with PSR option disabled",
                    indoc! {r#"
                    <?php

                    interface My {}
                "#},
                )
                .with_option(PSR, Value::Boolean(false)),
            )
            .with_example(
                RuleUsageExample::invalid(
                    "An interface not suffixed with `Interface`, with PSR option enabled",
                    indoc! {r#"
                    <?php

                    interface My {}
                "#},
                )
                .with_option(PSR, Value::Boolean(true)),
            )
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Interface(interface) = node else { return LintDirective::default() };

        let mut issues = vec![];
        let name = context.lookup(&interface.name.value);
        let fqcn = context.lookup_name(&interface.name);

        if !mago_casing::is_class_case(name) {
            issues.push(
                Issue::new(context.level(), format!("Interface name `{name}` should be in class case."))
                    .with_annotations([
                        Annotation::primary(interface.name.span())
                            .with_message(format!("Interface `{name}` is declared here.")),
                        Annotation::secondary(interface.span())
                            .with_message(format!("Interface `{fqcn}` is defined here.")),
                    ])
                    .with_note(format!("The interface name `{name}` does not follow class naming convention."))
                    .with_help(format!(
                        "Consider renaming it to `{}` to adhere to the naming convention.",
                        mago_casing::to_class_case(name)
                    )),
            );
        }

        if context.option(PSR).and_then(|o| o.as_bool()).unwrap_or(PSR_DEFAULT) && !name.ends_with("Interface") {
            issues.push(
                Issue::new(context.level(), format!("interface name `{name}` should be suffixed with `Interface`."))
                    .with_annotations([
                        Annotation::primary(interface.name.span())
                            .with_message(format!("Interface `{name}` is declared here.")),
                        Annotation::secondary(interface.span())
                            .with_message(format!("Interface `{fqcn}` is defined here.")),
                    ])
                    .with_note(format!("The interface name `{name}` does not follow PSR naming convention."))
                    .with_help(format!(
                        "Consider renaming it to `{name}Interface` to adhere to the naming convention."
                    )),
            );
        }

        for issue in issues {
            context.report(issue);
        }

        LintDirective::Prune
    }
}
