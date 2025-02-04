use indoc::indoc;
use toml::Value;

use mago_ast::ast::*;
use mago_ast::Node;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::directive::LintDirective;
use crate::rule::Rule;

const THRESHOLD: &str = "threshold";
const THRESHOLD_DEFAULT: i64 = 10;

#[derive(Clone, Copy, Debug)]
pub struct TooManyPropertiesRule;

impl Rule for TooManyPropertiesRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Too Many Properties", Level::Error)
            .with_description(indoc! {r#"
                Detects class-like structures with too many properties.

                This rule checks the number of properties in classes, traits, and interfaces.
                If the number of properties exceeds a configurable threshold, an issue is reported.
            "#})
            .with_option(RuleOptionDefinition {
                name: THRESHOLD,
                r#type: "integer",
                description: "The maximum allowed number of properties before triggering an issue.",
                default: Value::Integer(THRESHOLD_DEFAULT),
            })
    }
    fn lint_node(&self, node: mago_ast::Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let (kind, members) = match node {
            Node::Class(class) => ("Class", class.members.as_slice()),
            Node::Trait(r#trait) => ("Trait", r#trait.members.as_slice()),
            Node::AnonymousClass(anonymous_class) => ("Class", anonymous_class.members.as_slice()),
            Node::Interface(interface) => ("Interface", interface.members.as_slice()),
            _ => {
                return LintDirective::default();
            }
        };

        let threshold = context.option(THRESHOLD).and_then(|o| o.as_integer()).unwrap_or(THRESHOLD_DEFAULT) as usize;
        let mut properties = 0;
        for member in members {
            let ClassLikeMember::Property(property) = member else {
                continue;
            };

            match property {
                Property::Plain(plain_property) => {
                    properties += plain_property.items.len();
                }
                Property::Hooked(_) => {
                    properties += 1;
                }
            }
        }

        if properties > threshold {
            context.report(
                Issue::new(context.level(), format!("{} has too many properties.", kind))
                    .with_annotation(Annotation::primary(node.span()).with_message(format!(
                        "{} has {} properties, which exceeds the threshold of {}.",
                        kind, properties, threshold
                    )))
                    .with_note("Having a large number of properties can make classes harder to understand and maintain.")
                    .with_help(
                        "Try reducing the number of properties, or consider grouping related properties into a single object.",
                    )
            );

            // If this structure has too many props, we don't need to check the nested structures.
            LintDirective::Prune
        } else {
            // Continue checking nested structures, if any.
            LintDirective::Continue
        }
    }
}
