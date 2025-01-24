use indoc::indoc;

use mago_ast::ast::*;
use mago_php_version::PHPVersion;
use mago_reflection::class_like::ClassLikeReflection;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RequireParameterTypeRule;

impl Rule for RequireParameterTypeRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Require Parameter Type", Level::Warning)
            .with_minimum_supported_php_version(PHPVersion::PHP70)
            .with_description(indoc! {"
                Detects parameters that are missing a type hint.
            "})
            .with_example(RuleUsageExample::valid(
                "A function with a parameter that has a type hint",
                indoc! {r#"
                    <?php

                    function foo(string $bar): void
                    {
                        // ...
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A function with a parameter that is missing a type hint",
                indoc! {r#"
                    <?php

                    function foo($bar): void
                    {
                        // ...
                    }
                "#},
            ))
    }
}

impl RequireParameterTypeRule {
    fn report(function_like_parameter: &FunctionLikeParameter, context: &mut LintContext<'_>) {
        if function_like_parameter.hint.is_some() {
            return;
        }

        let parameter_name = context.lookup(&function_like_parameter.variable.name);

        context.report(
            Issue::new(context.level(), format!("Parameter `{}` is missing a type hint.", parameter_name))
                .with_annotation(
                    Annotation::primary(function_like_parameter.span())
                        .with_message(format!("Parameter `{}` is declared here", parameter_name)),
                )
                .with_note("Type hints improve code readability and help prevent type-related errors.")
                .with_help(format!("Consider adding a type hint to parameter `{}`.", parameter_name)),
        );
    }

    fn report_class_like_members(
        reflection: &ClassLikeReflection,
        members: &[ClassLikeMember],
        context: &mut LintContext<'_>,
    ) {
        for member in members {
            let ClassLikeMember::Method(method) = member else {
                continue;
            };

            let Some(method_reflection) = reflection.get_method(&method.name.value) else {
                continue;
            };

            if method_reflection.is_overriding {
                // This method is overriding a method from a parent class.
                continue;
            }

            for parameter in method.parameter_list.parameters.iter() {
                Self::report(parameter, context);
            }
        }
    }
}

impl<'a> Walker<LintContext<'a>> for RequireParameterTypeRule {
    fn walk_in_function(&self, function: &Function, context: &mut LintContext<'a>) {
        for parameter in function.parameter_list.parameters.iter() {
            Self::report(parameter, context);
        }
    }

    fn walk_in_closure(&self, closure: &Closure, context: &mut LintContext<'a>) {
        for parameter in closure.parameter_list.parameters.iter() {
            Self::report(parameter, context);
        }
    }

    fn walk_in_arrow_function(&self, arrow_function: &ArrowFunction, context: &mut LintContext<'a>) {
        for parameter in arrow_function.parameter_list.parameters.iter() {
            Self::report(parameter, context);
        }
    }

    fn walk_in_interface(&self, interface: &Interface, context: &mut LintContext<'a>) {
        let name = context.semantics.names.get(&interface.name);
        let Some(reflection) = context.codebase.get_interface(context.interner, name) else {
            return;
        };

        Self::report_class_like_members(reflection, interface.members.as_slice(), context);
    }

    fn walk_in_class(&self, class: &Class, context: &mut LintContext<'a>) {
        let name = context.semantics.names.get(&class.name);
        let Some(reflection) = context.codebase.get_class(context.interner, name) else {
            return;
        };

        Self::report_class_like_members(reflection, class.members.as_slice(), context);
    }

    fn walk_in_enum(&self, r#enum: &Enum, context: &mut LintContext<'a>) {
        let name = context.semantics.names.get(&r#enum.name);
        let Some(reflection) = context.codebase.get_enum(context.interner, name) else {
            return;
        };

        Self::report_class_like_members(reflection, r#enum.members.as_slice(), context);
    }

    fn walk_in_trait(&self, r#trait: &Trait, context: &mut LintContext<'a>) {
        let name = context.semantics.names.get(&r#trait.name);
        let Some(reflection) = context.codebase.get_trait(context.interner, name) else {
            return;
        };

        Self::report_class_like_members(reflection, r#trait.members.as_slice(), context);
    }
}
