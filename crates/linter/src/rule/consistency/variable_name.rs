use indoc::indoc;
use mago_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_casing::is_camel_case;
use mago_casing::is_snake_case;
use mago_casing::to_camel_case;
use mago_casing::to_snake_case;
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
pub struct VariableNameRule {
    meta: &'static RuleMeta,
    cfg: VariableNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct VariableNameConfig {
    pub level: Level,
    pub camel: bool,
    pub either: bool,
    pub check_parameters: bool,
    pub check_properties: bool,
    pub check_promoted_properties: bool,
}

impl Default for VariableNameConfig {
    fn default() -> Self {
        Self {
            level: Level::Help,
            camel: false,
            either: false,
            check_parameters: true,
            check_properties: true,
            check_promoted_properties: true,
        }
    }
}

impl Config for VariableNameConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for VariableNameRule {
    type Config = VariableNameConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Variable Name",
            code: "variable-name",
            description: indoc! {"
                Detects variable declarations, method parameters and class properties that do not follow camel or snake naming convention.

                Variable names should be in camel case or snake case, depending on the configuration.
            "},
            good_example: indoc! {r#"
                <?php

                $my_variable = 1;

                final class Foo {
                    public string $my_property;

                    public function __construct(
                        public int $my_promoted_property,
                    ) {}
                }

                function foo($my_param) {}
            "#},
            bad_example: indoc! {r#"
                <?php

                $MyVariable = 1;

                $My_Variable = 2;

                final class Foo {
                    public string $MyProperty;

                    public function __construct(
                        public int $MyPromotedProperty,
                    ) {}
                }

                function foo($MyParam) {}
            "#},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Assignment, NodeKind::FunctionLikeParameter, NodeKind::PropertyItem];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        match node {
            Node::Assignment(assignment) => {
                self.check_assignment(ctx, assignment);
            }
            Node::FunctionLikeParameter(parameter) => {
                self.check_parameter(ctx, parameter);
            }
            Node::PropertyItem(property_item) => {
                self.check_property_item(ctx, property_item);
            }
            _ => {}
        }
    }
}

impl VariableNameRule {
    fn check_assignment<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, assignment: &Assignment<'arena>) {
        if !assignment.operator.is_assign() {
            return;
        }

        let Expression::Variable(Variable::Direct(variable)) = assignment.lhs else {
            return;
        };

        let name = variable.name;

        if is_special_variable(name) {
            return;
        }

        self.check_variable_name(ctx, name, variable.span());
    }

    fn check_parameter<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, parameter: &FunctionLikeParameter<'arena>) {
        if parameter.is_promoted_property() && !self.cfg.check_promoted_properties {
            return;
        }

        if !parameter.is_promoted_property() && !self.cfg.check_parameters {
            return;
        }

        self.check_variable_name(ctx, parameter.variable.name, parameter.variable.span());
    }

    fn check_property_item<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, property_item: &PropertyItem<'arena>) {
        if !self.cfg.check_properties {
            return;
        }

        let variable = property_item.variable();

        self.check_variable_name(ctx, variable.name, variable.span());
    }

    fn check_variable_name(&self, ctx: &mut LintContext<'_, '_>, name: &str, span: Span) {
        let name_without_dollar = name.strip_prefix('$').unwrap_or(name);

        if self.cfg.either {
            if !is_camel_case(name_without_dollar) && !is_snake_case(name_without_dollar) {
                ctx.collector.report(
                    Issue::new(
                        self.cfg.level(),
                        format!("Variable name `{}` should be in either camel case or snake case.", name),
                    )
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(span).with_message(format!("Variable `{}` is declared here", name)),
                    )
                    .with_note(format!(
                        "The variable name `{}` does not follow either camel case or snake naming convention.",
                        name
                    ))
                    .with_help(format!(
                        "Consider renaming it to `${}` or `${}` to adhere to the naming convention.",
                        to_camel_case(name_without_dollar),
                        to_snake_case(name_without_dollar)
                    )),
                );
            }

            return;
        }

        if self.cfg.camel && !is_camel_case(name_without_dollar) {
            ctx.collector.report(
                Issue::new(self.cfg.level(), format!("Variable name `{}` should be in camel case.", name))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(span).with_message(format!("Variable `{}` is declared here", name)),
                    )
                    .with_note(format!("The variable name `{}` does not follow camel naming convention.", name))
                    .with_help(format!(
                        "Consider renaming it to `${}` to adhere to the naming convention.",
                        to_camel_case(name_without_dollar)
                    )),
            );
        } else if !self.cfg.camel && !is_snake_case(name_without_dollar) {
            ctx.collector.report(
                Issue::new(self.cfg.level(), format!("Variable name `{}` should be in snake case.", name))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(span).with_message(format!("Variable `{}` is declared here", name)),
                    )
                    .with_note(format!("The variable name `{}` does not follow snake naming convention.", name))
                    .with_help(format!(
                        "Consider renaming it to `${}` to adhere to the naming convention.",
                        to_snake_case(name_without_dollar)
                    )),
            );
        }
    }
}

fn is_special_variable(name: &str) -> bool {
    matches!(
        name,
        "$this"
            | "$_GET"
            | "$_POST"
            | "$_SERVER"
            | "$_REQUEST"
            | "$_SESSION"
            | "$_COOKIE"
            | "$_FILES"
            | "$_ENV"
            | "$GLOBALS"
            | "$argc"
            | "$argv"
    )
}
