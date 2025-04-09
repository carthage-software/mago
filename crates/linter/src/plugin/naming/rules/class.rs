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
pub struct ClassRule;

impl Rule for ClassRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Class", Level::Help)
            .with_description(indoc! {"
                Detects class declarations that do not follow class naming convention.
                Class names should be in class case, also known as PascalCase.
            "})
            .with_option(RuleOptionDefinition {
                name: PSR,
                r#type: "boolean",
                description:
                    "Enforce PSR naming convention, which requires abstract classes to be prefixed with `Abstract`.",
                default: Value::Boolean(PSR_DEFAULT),
            })
            .with_example(RuleUsageExample::valid(
                "A class name in class case",
                indoc! {r#"
                    <?php

                    class MyClass {}
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A class name not in class case",
                indoc! {r#"
                    <?php

                    class my_class {}
                    class myClass {}
                    class MY_CLASS {}
                "#},
            ))
            .with_example(
                RuleUsageExample::valid(
                    "An abstract class name not prefixed with `Abstract`, with PSR option disabled",
                    indoc! {r#"
                    <?php

                    abstract class MyClass {}
                "#},
                )
                .with_option(PSR, Value::Boolean(false)),
            )
            .with_example(
                RuleUsageExample::invalid(
                    "An abstract class name not prefixed with `Abstract`, with PSR option enabled",
                    indoc! {r#"
                    <?php

                    abstract class MyClass {}
                "#},
                )
                .with_option(PSR, Value::Boolean(true)),
            )
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Class(class) = node else { return LintDirective::default() };
        let mut issues = vec![];
        let name = context.lookup(&class.name.value);
        if !mago_casing::is_class_case(name) {
            let issue = Issue::new(context.level(), format!("Class name `{}` should be in class case.", name))
                .with_annotations([
                    Annotation::primary(class.name.span()).with_message(format!("Class `{}` is declared here.", name))
                ])
                .with_note(format!("The class name `{}` does not follow class naming convention.", name))
                .with_help(format!(
                    "Consider renaming it to `{}` to adhere to the naming convention.",
                    mago_casing::to_class_case(name)
                ));

            issues.push(issue);
        }

        if class.modifiers.contains_abstract()
            && context.option(PSR).and_then(|o| o.as_bool()).unwrap_or(PSR_DEFAULT)
            && !name.starts_with("Abstract")
        {
            let suggested_name = format!("Abstract{}", mago_casing::to_class_case(name));

            issues.push(
                Issue::new(
                    context.level(),
                    format!("Abstract class name `{}` should be prefixed with `Abstract`.", name),
                )
                .with_annotations([
                    Annotation::primary(class.name.span()).with_message(format!("Class `{}` is declared here.", name))
                ])
                .with_note(format!("The abstract class name `{}` does not follow PSR naming convention.", name))
                .with_help(format!("Consider renaming it to `{}` to adhere to the naming convention.", suggested_name)),
            );
        }

        for issue in issues {
            context.report(issue);
        }

        LintDirective::default()
    }
}
