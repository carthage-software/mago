use indoc::indoc;
use toml::Value;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

const THRESHOLD: &str = "threshold";
const THRESHOLD_DEFAULT: i64 = 10;

#[derive(Clone, Debug)]
pub struct LongInheritanceChainRule;

impl Rule for LongInheritanceChainRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Long Inheritance Chain", Level::Warning)
            .with_description(indoc! {"
            "})
            .with_option(RuleOptionDefinition {
                name: THRESHOLD,
                r#type: "integer",
                description: "The maximum allowed number of parents before triggering an issue.",
                default: Value::Integer(THRESHOLD_DEFAULT),
            })
            .with_example(
                RuleUsageExample::invalid(
                    "Class with 7 parent dependencies",
                    indoc! {r#"
                        <?php

                        interface A {}
                        interface B extends A {}
                        interface C {}
                        interface D extends C {}
                        interface E {}
                        interface F extends E, D, A {}

                        class Parent implements B {}

                        class Child extends Parent implements F {
                            // Total parents: 7 (Parent + B + F + E + D + C + A)
                        }
                    "#},
                )
                .with_option(THRESHOLD, Value::Integer(5)),
            )
            .with_example(
                RuleUsageExample::valid(
                    "Class with 2 parent dependencies",
                    indoc! {r#"
                        <?php

                        class SimpleParent {}
                        interface Loggable {}

                        class Child extends SimpleParent implements Loggable {
                            // Total parents: 2 (SimpleParent + Loggable)
                        }
                    "#},
                )
                .with_option(THRESHOLD, Value::Integer(5)),
            )
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let reflection = match node {
            Node::Class(class) => {
                let name = context.module.names.get(&class.name);
                let Some(reflection) = context.codebase.get_class(context.interner, name) else {
                    return LintDirective::default();
                };

                reflection
            }
            Node::Enum(r#enum) => {
                let name = context.module.names.get(&r#enum.name);
                let Some(reflection) = context.codebase.get_enum(context.interner, name) else {
                    return LintDirective::default();
                };

                reflection
            }
            Node::Interface(r#interface) => {
                let name = context.module.names.get(&r#interface.name);
                let Some(reflection) = context.codebase.get_interface(context.interner, name) else {
                    return LintDirective::default();
                };

                reflection
            }
            Node::Trait(r#trait) => {
                let name = context.module.names.get(&r#trait.name);
                let Some(reflection) = context.codebase.get_trait(context.interner, name) else {
                    return LintDirective::default();
                };

                reflection
            }
            Node::AnonymousClass(anonymous_class) => {
                let Some(reflection) = context.codebase.get_anonymous_class(&anonymous_class) else {
                    return LintDirective::default();
                };

                reflection
            }
            _ => return LintDirective::default(),
        };

        let parents = reflection.inheritance.all_extended_classes.len()
            + reflection.inheritance.all_extended_interfaces.len()
            + reflection.inheritance.all_implemented_interfaces.len();

        let threshold = context.option(THRESHOLD).and_then(|o| o.as_integer()).unwrap_or(THRESHOLD_DEFAULT) as usize;

        if parents > threshold {
            let issue = Issue::new(context.level(), "Inheritance chain is too long.".to_string())
                .with_annotation(Annotation::primary(reflection.span()).with_message(format!(
                    "{} `{}` has {} parents, which exceeds the threshold of {}.",
                    reflection.name.get_kind(),
                    reflection.name.get_key(context.interner),
                    parents,
                    threshold
                )))
                .with_note("Having a large number of parents can make classes harder to understand and maintain.")
                .with_help("Try reducing the number of parents, or consider using composition instead.");

            context.report(issue);
        }

        LintDirective::default()
    }
}
