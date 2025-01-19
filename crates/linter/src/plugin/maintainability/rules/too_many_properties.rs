use indoc::indoc;
use toml::Value;

use mago_ast::ast::*;
use mago_ast::Node;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
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
}

impl<'a> Walker<LintContext<'a>> for TooManyPropertiesRule {
    fn walk_in_class(&self, class: &Class, context: &mut LintContext<'a>) {
        check("Class", Node::Class(class), class.members.as_slice(), context);
    }

    fn walk_in_trait(&self, r#trait: &Trait, context: &mut LintContext<'a>) {
        check("Trait", Node::Trait(r#trait), r#trait.members.as_slice(), context);
    }

    fn walk_in_interface(&self, interface: &Interface, context: &mut LintContext<'a>) {
        check("Interface", Node::Interface(interface), interface.members.as_slice(), context);
    }

    fn walk_in_anonymous_class(&self, anonymous_class: &AnonymousClass, context: &mut LintContext<'a>) {
        check("Class", Node::AnonymousClass(anonymous_class), anonymous_class.members.as_slice(), context);
    }
}

#[inline]
fn check(kind: &'static str, node: Node<'_>, members: &[ClassLikeMember], context: &mut LintContext<'_>) {
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
        let issue = Issue::new(context.level(), format!("{} has too many properties.", kind))
            .with_annotation(Annotation::primary(node.span()).with_message(format!(
                "{} has {} properties, which exceeds the threshold of {}.",
                kind, properties, threshold
            )))
            .with_note("Having a large number of properties can make classes harder to understand and maintain.")
            .with_help(
                "Try reducing the number of properties, or consider grouping related properties into a single object.",
            );

        context.report(issue);
    }
}
