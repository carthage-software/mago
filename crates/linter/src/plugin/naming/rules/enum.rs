use indoc::indoc;

use mago_ast::ast::*;
use mago_reporting::*;
use mago_span::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::rule::Rule;

#[derive(Clone, Copy, Debug)]
pub struct EnumRule;

impl Rule for EnumRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Enum", Level::Help)
            .with_description(indoc! {"
                Detects enum declarations that do not follow class naming convention.
                Enum names should be in class case, also known as PascalCase.
            "})
            .with_example(RuleUsageExample::valid(
                "An enum name in class case",
                indoc! {r#"
                    <?php

                    enum MyEnum {}
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "An enum name not in class case",
                indoc! {r#"
                    <?php

                    enum my_enum {}
                    enum myEnum {}
                    enum MY_ENUM {}
                "#},
            ))
    }
}

impl<'a> Walker<LintContext<'a>> for EnumRule {
    fn walk_in_enum<'ast>(&self, r#enum: &'ast Enum, context: &mut LintContext<'a>) {
        let name = context.lookup(&r#enum.name.value);
        let fqcn = context.lookup_name(&r#enum.name);

        if !mago_casing::is_class_case(name) {
            context.report(
                Issue::new(context.level(), format!("Enum name `{}` should be in class case.", name))
                    .with_annotation(
                        Annotation::primary(r#enum.name.span())
                            .with_message(format!("Enum `{}` is declared here.", name)),
                    )
                    .with_annotation(
                        Annotation::secondary(r#enum.span()).with_message(format!("Enum `{}` is defined here.", fqcn)),
                    )
                    .with_note(format!("The enum name `{}` does not follow class naming convention.", name))
                    .with_help(format!(
                        "Consider renaming it to `{}` to adhere to the naming convention.",
                        mago_casing::to_class_case(name)
                    )),
            );
        }
    }
}
