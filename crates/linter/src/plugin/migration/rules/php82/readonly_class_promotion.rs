use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct ReadonlyClassPromotionRule;

impl Rule for ReadonlyClassPromotionRule {
    fn get_name(&self) -> &'static str {
        "readonly-class-promotion"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Warning)
    }
}

impl<'a> Walker<LintContext<'a>> for ReadonlyClassPromotionRule {
    fn walk_in_class(&self, class: &Class, context: &mut LintContext<'a>) {
        let name = context.semantics.names.get(&class.name);
        let Some(reflection) = context.codebase.get_class(context.interner, name) else {
            return;
        };

        // If the class is readonly, extends another class or has children, we can't promote it.
        if reflection.is_readonly || reflection.inheritance.extends_classes() || reflection.inheritance.has_children() {
            return;
        }

        if !reflection.is_final && context.option("final-only").and_then(|c| c.as_bool()).unwrap_or(false) {
            return;
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
                    for param in method.parameters.parameters.iter() {
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
            return;
        }

        let annotations = class
            .members
            .iter()
            .filter_map(|member| {
                if let ClassLikeMember::Property(property) = member {
                    property.modifiers().get_readonly().map(|modifier| {
                        Annotation::secondary(modifier.span()).with_message("property is marked as readonly")
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
                        for param in method.parameters.parameters.iter() {
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
    }
}
