use indoc::indoc;

use mago_fixer::SafetyClassification;
use mago_interner::StringIdentifier;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
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

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Method(method) = node else { return LintDirective::default() };

        let MethodBody::Concrete(block) = &method.body else {
            return LintDirective::Prune;
        };

        if block.statements.len() != 1 {
            return LintDirective::default();
        }

        let name = method.name.value;
        let parameters = method
            .parameter_list
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
            _ => return LintDirective::default(),
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

            context
                .propose(issue, |plan| plan.delete(method.span().to_range(), SafetyClassification::PotentiallyUnsafe));
        }

        LintDirective::Prune
    }
}

fn matches_method(
    method_name: &StringIdentifier,
    parameters: &[(bool, StringIdentifier)],
    expression: &Expression,
) -> bool {
    let Expression::Call(Call::StaticMethod(StaticMethodCall { class, method, argument_list: arguments, .. })) =
        expression
    else {
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
            Argument::Named(arg) => (false, &arg.value),
        };

        if variadic.eq(is_variadic)
            || !matches!(value, Expression::Variable(Variable::Direct(variable)) if variable.name.eq(parameter))
        {
            return false;
        }
    }

    true
}
