use indoc::indoc;
use mago_php_version::PHPVersion;
use toml::Value;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

const FINAL_ONLY: &str = "final-only";
const FINAL_ONLY_DEFAULT: bool = false;

#[derive(Clone, Debug)]
pub struct ReadonlyClassPromotionRule;

impl Rule for ReadonlyClassPromotionRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Readonly Class Promotion", Level::Warning)
            .with_minimum_supported_php_version(PHPVersion::PHP82)
            .with_description(indoc! {"
                Detects classes that contain only readonly properties and suggests promoting them to readonly classes.
                Promoting classes to readonly classes can improve code readability and intent clarity.
            "})
            .with_option(RuleOptionDefinition {
                name: FINAL_ONLY,
                r#type: "boolean",
                description: "Only promote final classes",
                default: Value::Boolean(FINAL_ONLY_DEFAULT),
            })
            .with_example(RuleUsageExample::valid(
                "Class is already readonly",
                indoc! {r#"
                    <?php

                    readonly class Foo {
                        public int $a;
                        public int $b;
                    }
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Class does not contain all readonly properties",
                indoc! {r#"
                    <?php

                    class Foo {
                        public readonly int $a;
                        public int $b;
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Class contains only readonly properties, but is not marked as readonly",
                indoc! {r#"
                    <?php

                    class Foo {
                        public readonly int $a;
                        public readonly int $b;
                    }
                "#},
            ))
            .with_example(
                RuleUsageExample::valid(
                    "Class contains only readonly properties, but is not final, and final-only option is enabled",
                    indoc! {r#"
                    <?php

                    class Foo {
                        public readonly int $a;
                        public readonly int $b;
                    }
                "#},
                )
                .with_option(FINAL_ONLY, Value::Boolean(true)),
            )
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Class(class) = node else { return LintDirective::default() };

        let name = context.semantics.names.get(&class.name);
        let Some(reflection) = context.codebase.get_class(context.interner, name) else {
            return LintDirective::default();
        };

        // If the class is readonly, extends another class or has children, we can't promote it.
        if reflection.is_readonly || reflection.inheritance.extends_classes() || reflection.inheritance.has_children() {
            return LintDirective::default();
        }

        if !reflection.is_final && context.option(FINAL_ONLY).and_then(|c| c.as_bool()).unwrap_or(FINAL_ONLY_DEFAULT) {
            return LintDirective::default();
        }

        let mut all_properties_readonly = true;
        let mut property_count = 0;
        for member in class.members.iter() {
            match member {
                ClassLikeMember::Property(property) => {
                    property_count += 1;
                    if !property.modifiers().contains_readonly() {
                        all_properties_readonly = false;
                        break;
                    }
                }
                ClassLikeMember::Method(method) => {
                    for param in method.parameter_list.parameters.iter() {
                        if param.is_promoted_property() {
                            property_count += 1;
                            if !param.modifiers.contains_readonly() {
                                all_properties_readonly = false;
                                break;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if !all_properties_readonly || property_count == 0 {
            return LintDirective::default();
        }

        let annotations = class
            .members
            .iter()
            .filter_map(|member| {
                if let ClassLikeMember::Property(property) = member {
                    property.modifiers().get_readonly().map(|modifier| {
                        Annotation::secondary(modifier.span()).with_message("Property is marked as readonly.")
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Prepare fix plan
        let issue = Issue::new(context.level(), "Promote class to readonly")
            .with_annotations(annotations)
            .with_annotation(
                Annotation::primary(class.span()).with_message("This class contains only readonly properties."),
            )
            .with_note("Classes that contains only readonly properties can be marked readonly themselves.")
            .with_help("Add the `readonly` modifier to the class and remove `readonly` from all properties");

        context.report_with_fix(issue, |plan| {
            // Remove readonly from all properties
            for member in class.members.iter() {
                match member {
                    ClassLikeMember::Property(property) => {
                        if let Some(readonly) = property.modifiers().get_readonly() {
                            plan.delete(readonly.span().to_range(), SafetyClassification::Safe);
                        }
                    }
                    ClassLikeMember::Method(method) => {
                        for param in method.parameter_list.parameters.iter() {
                            if param.is_promoted_property() {
                                if let Some(readonly) = param.modifiers.get_readonly() {
                                    plan.delete(readonly.span().to_range(), SafetyClassification::Safe);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Add readonly keyword to the class
            plan.insert(class.class.span.start_position().offset, "readonly ", SafetyClassification::Safe);
        });

        LintDirective::default()
    }
}
