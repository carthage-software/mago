use indoc::indoc;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_interner::StringIdentifier;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RedundantMethodOverrideRule;

impl Rule for RedundantMethodOverrideRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Redundant Method Override", Level::Help)
            .with_description(indoc! {"
                Detects methods that override a parent method but only call the parent method with the same arguments.
            "})
            .with_example(RuleUsageExample::invalid(
                "A method that overrides a parent method but only calls the parent method with the same arguments",
                indoc! {r#"
                    <?php

                    class Parent
                    {
                        public function foo(): void
                        {
                            // ...
                        }
                    }

                    class Child extends Parent
                    {
                        public function foo(): void
                        {
                            parent::foo();
                        }
                    }
                "#},
            ))
    }
}

impl<'a> Walker<LintContext<'a>> for RedundantMethodOverrideRule {
    fn walk_in_method<'ast>(&self, method: &'ast Method, context: &mut LintContext<'a>) {
        let MethodBody::Concrete(block) = &method.body else {
            return;
        };

        if block.statements.len() != 1 {
            return;
        }

        let name = method.name.value;
        let parameters = method
            .parameters
            .parameters
            .iter()
            .map(|parameter| (parameter.ellipsis.is_some(), parameter.variable.name))
            .collect::<Vec<_>>();

        let statement = block
            .statements
            .first()
            .expect("Method body is guaranteed to have at least one statement, so this unwrap is safe");

        let expression = match &statement {
            Statement::Return(Return { value: Some(expression), .. }) => expression,
            Statement::Expression(ExpressionStatement { expression, .. }) => expression,
            _ => return,
        };

        if matches_method(&name, &parameters, expression) {
            let issue = Issue::new(context.level(), "Redundant method override.")
                .with_annotation(Annotation::primary(method.span()))
                .with_annotation(
                    Annotation::secondary(expression.span())
                        .with_message("Parent method is called with the same arguments."),
                )
                .with_note(
                    "This method overrides a parent method but only calls the parent method with the same arguments.",
                )
                .with_help("Remove this redundant method override.");

            context.report_with_fix(issue, |plan| {
                plan.delete(method.span().to_range(), SafetyClassification::PotentiallyUnsafe)
            });
        }
    }
}

fn matches_method(
    method_name: &StringIdentifier,
    parameters: &[(bool, StringIdentifier)],
    expression: &Expression,
) -> bool {
    let Expression::Call(Call::StaticMethod(StaticMethodCall { class, method, arguments, .. })) = expression else {
        return false;
    };

    if !matches!(class.as_ref(), Expression::Parent(_))
        || !matches!(method, ClassLikeMemberSelector::Identifier(identifier) if identifier.value.eq(method_name))
        || arguments.arguments.len() != parameters.len()
    {
        return false;
    }

    for (argument, (is_variadic, parameter)) in arguments.arguments.iter().zip(parameters.iter()) {
        let (variadic, value) = match &argument {
            Argument::Positional(arg) => (arg.ellipsis.is_some(), &arg.value),
            Argument::Named(arg) => (arg.ellipsis.is_some(), &arg.value),
        };

        if variadic.eq(is_variadic)
            || !matches!(value, Expression::Variable(Variable::Direct(variable)) if variable.name.eq(parameter))
        {
            return false;
        }
    }

    true
}
