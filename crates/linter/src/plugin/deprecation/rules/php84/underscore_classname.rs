use indoc::indoc;

use mago_ast::*;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct UnderscoreClassNameRule;

impl Rule for UnderscoreClassNameRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Underscore Class Name", Level::Warning)
            .with_description(indoc! {"
                    Detects class, interface, trait, or enum declarations named `_`.
                    Such names are considered deprecated; a more descriptive identifier is recommended.
                "})
            .with_example(RuleUsageExample::valid(
                "Using a meaningful class name",
                indoc! {r#"
                    <?php

                    class MyService {}
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `_` as a class name",
                indoc! {r#"
                    <?php

                    class _ {}
                "#},
            ))
    }
}

impl<'a> Walker<LintContext<'a>> for UnderscoreClassNameRule {
    fn walk_in_class(&self, class: &Class, context: &mut LintContext<'a>) {
        let class_name = context.lookup(&class.name.value);
        if class_name != "_" {
            return;
        }

        let issue = Issue::new(context.level(), "Using `_` as a class name is deprecated.")
            .with_annotation(
                Annotation::primary(class.name.span()).with_message("Rename the class to something more descriptive."),
            )
            .with_note("Class names consisting only of `_` are deprecated. consider using a meaningful name.");

        context.report(issue);
    }

    fn walk_in_interface(&self, interface: &Interface, context: &mut LintContext<'a>) {
        let interface_name = context.lookup(&interface.name.value);
        if interface_name != "_" {
            return;
        }

        let issue = Issue::new(context.level(), "Using `_` as an interface name is deprecated.")
            .with_annotation(
                Annotation::primary(interface.name.span())
                    .with_message("Rename the interface to something more descriptive."),
            )
            .with_note("Interface names consisting only of `_` are deprecated. consider using a meaningful name.");

        context.report(issue);
    }

    fn walk_in_trait(&self, r#trait: &Trait, context: &mut LintContext<'a>) {
        let trait_name = context.lookup(&r#trait.name.value);
        if trait_name != "_" {
            return;
        }

        let issue = Issue::new(context.level(), "Using `_` as a trait name is deprecated.")
            .with_annotation(
                Annotation::primary(r#trait.name.span())
                    .with_message("Rename the trait to something more descriptive."),
            )
            .with_note("Trait names consisting only of `_` are deprecated. consider using a meaningful name.");

        context.report(issue);
    }

    fn walk_in_enum(&self, r#enum: &Enum, context: &mut LintContext<'a>) {
        let enum_name = context.lookup(&r#enum.name.value);
        if enum_name != "_" {
            return;
        }

        let issue = Issue::new(context.level(), "Using `_` as an enum name is deprecated.")
            .with_annotation(
                Annotation::primary(r#enum.name.span()).with_message("Rename the enum to something more descriptive."),
            )
            .with_note("Enum names consisting only of `_` are deprecated. consider using a meaningful name.");

        context.report(issue);
    }
}
