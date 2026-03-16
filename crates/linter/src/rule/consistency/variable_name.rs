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
        let clean_name = name.trim_start_matches(|c: char| c == '$' || c == '_' || c.is_ascii_digit());

        if self.cfg.either {
            if !is_camel_case(clean_name) && !is_snake_case(clean_name) {
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
                        to_camel_case(clean_name),
                        to_snake_case(clean_name)
                    )),
                );
            }

            return;
        }

        if self.cfg.camel && !is_camel_case(clean_name) {
            ctx.collector.report(
                Issue::new(self.cfg.level(), format!("Variable name `{name}` should be in camel case."))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(span).with_message(format!("Variable `{name}` is declared here")),
                    )
                    .with_note(format!("The variable name `{name}` does not follow camel naming convention."))
                    .with_help(format!(
                        "Consider renaming it to `${}` to adhere to the naming convention.",
                        to_camel_case(clean_name)
                    )),
            );
        } else if !self.cfg.camel && !is_snake_case(clean_name) {
            ctx.collector.report(
                Issue::new(self.cfg.level(), format!("Variable name `{name}` should be in snake case."))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(span).with_message(format!("Variable `{name}` is declared here")),
                    )
                    .with_note(format!("The variable name `{name}` does not follow snake naming convention."))
                    .with_help(format!(
                        "Consider renaming it to `${}` to adhere to the naming convention.",
                        to_snake_case(clean_name)
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

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    use super::*;

    test_lint_success! {
        name = snake_case_variable_success,
        rule = VariableNameRule,
        settings = |s: &mut Settings| {
            s.rules.variable_name.config.camel = false;
            s.rules.variable_name.config.either = false;
        },
        code = indoc! {r#"
            <?php

            $_ = 1;
            $_foo_bar = 1;
            $bar_baz = 2;
            function foo($qux) {}
        "#}
    }

    test_lint_success! {
        name = camel_case_success,
        rule = VariableNameRule,
        settings = |s: &mut Settings| {
            s.rules.variable_name.config.camel = true;
            s.rules.variable_name.config.either = false;
        },
        code = "<?php $myVariableName = 1; $simple = 2;"
    }

    test_lint_success! {
        name = either_case_success,
        rule = VariableNameRule,
        settings = |s: &mut Settings| {
            s.rules.variable_name.config.either = true;
        },
        code = "<?php $snake_case = 1; $camelCase = 2;"
    }

    test_lint_success! {
        name = leading_special_chars_success,
        rule = VariableNameRule,
        settings = |s: &mut Settings| { s.rules.variable_name.config.camel = false; },
        code = "<?php $__hidden_variable = 1; $v1_name = 2;"
    }

    test_lint_success! {
        name = special_variables_ignored,
        rule = VariableNameRule,
        code = "<?php $this->foo = $_GET; $GLOBALS['x'] = 1; $argv = [];"
    }

    test_lint_success! {
        name = parameter_naming_success,
        rule = VariableNameRule,
        settings = |s: &mut Settings| { s.rules.variable_name.config.camel = false; },
        code = "<?php function test($valid_param, $another_one) {}"
    }

    test_lint_success! {
        name = skip_parameter_checks,
        rule = VariableNameRule,
        settings = |s: &mut Settings| {
            s.rules.variable_name.config.check_parameters = false;
            s.rules.variable_name.config.camel = true;
        },
        code = "<?php function test($bad_snake_param) {}"
    }

    test_lint_success! {
        name = promoted_properties_ignored,
        rule = VariableNameRule,
        code = "<?php class Foo { public function __construct(public int $some_Property) {} }"
    }

    test_lint_success! {
        name = complex_assignments_ignored,
        rule = VariableNameRule,
        code = "<?php $obj->SomeProp = 1; $$dynamic_Var = 2;"
    }

    test_lint_failure! {
        name = snake_case_variable_failure,
        rule = VariableNameRule,
        count = 2,
        settings = |s: &mut Settings| {
            s.rules.variable_name.config.camel = false;
            s.rules.variable_name.config.either = false;
        },
        code = indoc! {r#"
            <?php

            $_ = 1; // OK
            $_fooBar = 1;
            $barBaz = 2;
            function foo($qux) {} // ok
        "#}
    }

    test_lint_failure! {
        name = camel_case_failure,
        rule = VariableNameRule,
        count = 1,
        settings = |s: &mut Settings| {
            s.rules.variable_name.config.camel = true;
            s.rules.variable_name.config.either = false;
        },
        code = "<?php $my_variable_name = 1;"
    }

    test_lint_failure! {
        name = pascal_case_failure,
        rule = VariableNameRule,
        count = 1,
        settings = |s: &mut Settings| { s.rules.variable_name.config.either = true; },
        code = "<?php $MyVariable = 1;"
    }
}
