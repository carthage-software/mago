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
use mago_syntax::ast::Assignment;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionLikeParameter;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Variable;

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
}

impl Default for VariableNameConfig {
    fn default() -> Self {
        Self { level: Level::Help, camel: false, either: true, check_parameters: true }
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
                Detects variable declarations that do not follow camel or snake naming convention.

                Variable names should be in camel case or snake case, depending on the configuration.
            "},
            good_example: indoc! {r"
                <?php

                $my_variable = 1;

                function foo($my_param) {}
            "},
            bad_example: indoc! {r"
                <?php

                $MyVariable = 1;

                $My_Variable = 2;

                function foo($MyParam) {}
            "},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Assignment, NodeKind::FunctionLikeParameter];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        match node {
            Node::Assignment(assignment) => {
                self.check_assignment(ctx, assignment);
            }
            Node::FunctionLikeParameter(parameter) => {
                self.check_parameter(ctx, parameter);
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
        if !self.cfg.check_parameters {
            return;
        }

        if parameter.is_promoted_property() {
            return;
        }

        let name = parameter.variable.name;

        self.check_variable_name(ctx, name, parameter.variable.span());
    }

    fn check_variable_name(&self, ctx: &mut LintContext<'_, '_>, name: &str, span: Span) {
        let name_without_dollar = name.strip_prefix('$').unwrap_or(name);

        if self.cfg.either {
            if !is_camel_case(name_without_dollar) && !is_snake_case(name_without_dollar) {
                ctx.collector.report(
                    Issue::new(
                        self.cfg.level(),
                        format!("Variable name `{name}` should be in either camel case or snake case."),
                    )
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(span).with_message(format!("Variable `{name}` is declared here")),
                    )
                    .with_note(format!(
                        "The variable name `{name}` does not follow either camel case or snake naming convention."
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
                Issue::new(self.cfg.level(), format!("Variable name `{name}` should be in camel case."))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(span).with_message(format!("Variable `{name}` is declared here")),
                    )
                    .with_note(format!("The variable name `{name}` does not follow camel naming convention."))
                    .with_help(format!(
                        "Consider renaming it to `${}` to adhere to the naming convention.",
                        to_camel_case(name_without_dollar)
                    )),
            );
        } else if !self.cfg.camel && !is_snake_case(name_without_dollar) {
            ctx.collector.report(
                Issue::new(self.cfg.level(), format!("Variable name `{name}` should be in snake case."))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(span).with_message(format!("Variable `{name}` is declared here")),
                    )
                    .with_note(format!("The variable name `{name}` does not follow snake naming convention."))
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
            | "$_"
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
