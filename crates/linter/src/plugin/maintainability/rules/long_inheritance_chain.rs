use indoc::indoc;
use toml::Value;

use mago_codex::symbol::SymbolKind;
use mago_codex::*;
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
        let metadata = match node {
            Node::Class(class) => {
                let name = context.resolved_names.get(&class.name);

                get_class(context.codebase, context.interner, name)
            }
            Node::Enum(r#enum) => {
                let name = context.resolved_names.get(&r#enum.name);

                get_enum(context.codebase, context.interner, name)
            }
            Node::Interface(r#interface) => {
                let name = context.resolved_names.get(&r#interface.name);

                get_interface(context.codebase, context.interner, name)
            }
            Node::Trait(r#trait) => {
                let name = context.resolved_names.get(&r#trait.name);

                get_trait(context.codebase, context.interner, name)
            }
            Node::AnonymousClass(anonymous_class) => {
                get_anonymous_class(context.codebase, context.interner, anonymous_class.span())
            }
            _ => return LintDirective::default(),
        };

        let Some(metadata) = metadata else {
            return LintDirective::default();
        };

        let parents = metadata.all_parent_classes.len() + metadata.all_parent_interfaces.len();

        let threshold = context.option(THRESHOLD).and_then(|o| o.as_integer()).unwrap_or(THRESHOLD_DEFAULT) as usize;

        if parents > threshold {
            let issue = Issue::new(context.level(), "Inheritance chain is too long.".to_string())
                .with_annotation(Annotation::primary(metadata.span).with_message(format!(
                    "{} `{}` has {} parents, which exceeds the threshold of {}.",
                    match metadata.kind {
                        SymbolKind::Class => "Class",
                        SymbolKind::Enum => "Enum",
                        SymbolKind::Trait => "Trait",
                        SymbolKind::Interface => "Interface",
                    },
                    context.interner.lookup(&metadata.original_name),
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
