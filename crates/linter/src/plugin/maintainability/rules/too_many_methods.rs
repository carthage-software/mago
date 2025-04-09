use indoc::indoc;
use toml::Value;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::directive::LintDirective;
use crate::rule::Rule;

use crate::plugin::maintainability::rules::utils::is_method_setter_or_getter;

const THRESHOLD: &str = "threshold";
const THRESHOLD_DEFAULT: i64 = 10;

const COUNT_SETTERS_AND_GETTERS: &str = "count_setters_and_getters";
const COUNT_SETTERS_AND_GETTERS_DEFAULT: bool = false;

const COUNT_HOOKS: &str = "count_hooks";
const COUNT_HOOKS_DEFAULT: bool = false;

#[derive(Clone, Copy, Debug)]
pub struct TooManyMethodsRule;

impl Rule for TooManyMethodsRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Too Many Methods", Level::Error)
            .with_description(indoc! {r#"
                Detects class-like structures with too many methods.

                This rule checks the number of methods in classes, traits, enums, and interfaces.
                If the number of methods exceeds a configurable threshold, an issue is reported.
            "#})
            .with_option(RuleOptionDefinition {
                name: THRESHOLD,
                r#type: "integer",
                description: "The maximum allowed number of methods before triggering an issue.",
                default: Value::Integer(THRESHOLD_DEFAULT),
            })
            .with_option(RuleOptionDefinition {
                name: COUNT_SETTERS_AND_GETTERS,
                r#type: "boolean",
                description: "Whether to count setters and getters as methods.",
                default: Value::Boolean(COUNT_SETTERS_AND_GETTERS_DEFAULT),
            })
            .with_option(RuleOptionDefinition {
                name: COUNT_HOOKS,
                r#type: "boolean",
                description: "Whether to count property hooks as methods.",
                default: Value::Boolean(COUNT_HOOKS_DEFAULT),
            })
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let (kind, members) = match node {
            Node::Class(class) => ("Class", class.members.as_slice()),
            Node::Trait(r#trait) => ("Trait", r#trait.members.as_slice()),
            Node::AnonymousClass(anonymous_class) => ("Class", anonymous_class.members.as_slice()),
            Node::Enum(r#enum) => ("Enum", r#enum.members.as_slice()),
            Node::Interface(interface) => ("Interface", interface.members.as_slice()),
            _ => {
                return LintDirective::default();
            }
        };

        let threshold = context.option(THRESHOLD).and_then(|o| o.as_integer()).unwrap_or(THRESHOLD_DEFAULT) as usize;
        let count_hooks = context.option(COUNT_HOOKS).and_then(|o| o.as_bool()).unwrap_or(COUNT_HOOKS_DEFAULT);
        let count_setters_and_getters = context
            .option(COUNT_SETTERS_AND_GETTERS)
            .and_then(|o| o.as_bool())
            .unwrap_or(COUNT_SETTERS_AND_GETTERS_DEFAULT);

        let mut methods = 0;
        for member in members {
            match member {
                ClassLikeMember::Method(method) => {
                    if !count_setters_and_getters && is_method_setter_or_getter(method, context) {
                        continue;
                    }

                    methods += 1;
                }
                ClassLikeMember::Property(Property::Hooked(hooked_property)) if count_hooks => {
                    methods += hooked_property.hooks.hooks.len();
                }
                _ => (),
            }
        }

        if methods > threshold {
            context.report(Issue::new(context.level(), format!("{} has too many methods.", kind))
                .with_annotation(Annotation::primary(node.span()).with_message(format!(
                    "{} has {} methods, which exceeds the threshold of {}.",
                    kind, methods, threshold
                )))
                .with_note("Having a large number of methods can make classes harder to understand and maintain.")
                .with_help(
                    "Try reducing the number of methods, or consider splitting the structure into smaller, more focused structures.",
                ));

            // If this structure has too many methods, we don't need to check the nested structures.
            LintDirective::Prune
        } else {
            // Continue checking nested structures, if any.
            LintDirective::Continue
        }
    }
}
