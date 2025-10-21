use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoRedundantMethodOverrideRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantMethodOverrideConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantMethodOverrideConfig {
    pub level: Level,
}

impl Default for NoRedundantMethodOverrideConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoRedundantMethodOverrideConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantMethodOverrideRule {
    type Config = NoRedundantMethodOverrideConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Method Override",
            code: "no-redundant-method-override",
            description: indoc! {"
                Detects methods that override a parent method but only call the parent method with the same arguments.
            "},
            good_example: indoc! {r#"
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

                        echo 'Additional logic here';
                    }
                }
            "#},
            bad_example: indoc! {r#"
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
            category: Category::Redundancy,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Method];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Method(method) = node else {
            return;
        };

        let MethodBody::Concrete(block) = &method.body else {
            return;
        };

        if block.statements.len() != 1 {
            return;
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

        let expression = match statement {
            Statement::Return(Return { value: Some(expression), .. }) => expression,
            Statement::Expression(ExpressionStatement { expression, .. }) => expression,
            _ => return,
        };

        if matches_method(name, &parameters, expression) {
            let issue = Issue::new(self.cfg.level(), "Redundant method override.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(method.span()))
                .with_annotation(
                    Annotation::secondary(expression.span())
                        .with_message("Parent method is called with the same arguments"),
                )
                .with_note(
                    "This method overrides a parent method but only calls the parent method with the same arguments.",
                )
                .with_help("Remove this redundant method override.");

            ctx.collector.report(issue);
        }
    }
}

fn matches_method<'arena>(
    method_name: &'arena str,
    parameters: &[(bool, &'arena str)],
    expression: &Expression<'arena>,
) -> bool {
    let Expression::Call(Call::StaticMethod(StaticMethodCall { class, method, argument_list: arguments, .. })) =
        expression
    else {
        return false;
    };

    if !matches!(class, Expression::Parent(_))
        || !matches!(method, ClassLikeMemberSelector::Identifier(identifier) if identifier.value.eq(method_name))
        || arguments.arguments.len() != parameters.len()
    {
        return false;
    }

    for (argument, (is_variadic, parameter)) in arguments.arguments.iter().zip(parameters.iter()) {
        let (variadic, value) = match argument {
            Argument::Positional(arg) => (arg.ellipsis.is_some(), &arg.value),
            Argument::Named(arg) => (false, &arg.value),
        };

        if variadic.eq(is_variadic)
            || !matches!(value, Expression::Variable(Variable::Direct(variable)) if variable.name.eq(*parameter))
        {
            return false;
        }
    }

    true
}
