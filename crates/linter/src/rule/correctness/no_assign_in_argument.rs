use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Assignment;
use mago_syntax::ast::AssignmentOperator;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoAssignInArgumentRule {
    meta: &'static RuleMeta,
    cfg: NoAssignInArgumentConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoAssignInArgumentConfig {
    pub level: Level,
}

impl Default for NoAssignInArgumentConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoAssignInArgumentConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoAssignInArgumentRule {
    type Config = NoAssignInArgumentConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Assign In Argument",
            code: "no-assign-in-argument",
            description: indoc! {"
                Detects assignments in function call arguments which can lead to unexpected behavior and make
                the code harder to read and understand.
            "},
            good_example: indoc! {r"
                <?php

                $x = 5;
                foo($x);
            "},
            bad_example: indoc! {r"
                <?php

                foo($x = 5);
            "},
            category: Category::Correctness,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[
            NodeKind::ArgumentList,
        ];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::ArgumentList(argument_list) = node else {
          return;
        };

        for argument in argument_list.arguments.iter() {
            let value = argument.value();

            if let Some(assignment) = get_assignment_from_expression(value) {
                let mut issue = Issue::new(self.cfg.level(), "Avoid assignments in function call arguments.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(assignment.span()).with_message("This is an assignment"),
                    )
                    .with_annotation(
                        Annotation::secondary(argument_list.span()).with_message("In this argument list"),
                    )
                    .with_note("Assigning a value within a function call argument can lead to unexpected behavior and make the code harder to read and understand.")
                    .with_help("Consider assigning the variable before the function call.");

                if matches!(&assignment.operator, AssignmentOperator::Assign(_)) {
                    issue = issue.with_note("It's easy to confuse assignment (`=`) with comparison (`==`) in this context. Ensure you're using the correct operator.");
                }

                ctx.collector.report(issue);
            }
        }
    }
}

#[inline]
fn get_assignment_from_expression<'ast, 'arena>(
    expression: &'ast Expression<'arena>,
) -> Option<&'ast Assignment<'arena>> {
    match expression {
        Expression::Parenthesized(parenthesized) => get_assignment_from_expression(parenthesized.expression),
        Expression::Assignment(assignment) => Some(assignment),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoAssignInArgumentRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = assignment_in_function_argument,
        rule = NoAssignInArgumentRule,
        count = 1,
        code = indoc! {r"
            <?php

            foo($x = 5);
        "}
    }

    test_lint_failure! {
        name = assignment_in_method_argument,
        rule = NoAssignInArgumentRule,
        count = 1,
        code = indoc! {r"
            <?php

            $obj->method($x = getValue());
        "}
    }

    test_lint_failure! {
        name = assignment_in_static_method_argument,
        rule = NoAssignInArgumentRule,
        count = 1,
        code = indoc! {r"
            <?php

            MyClass::method($x = 10);
        "}
    }

    test_lint_failure! {
        name = assignment_in_null_safe_method_argument,
        rule = NoAssignInArgumentRule,
        count = 1,
        code = indoc! {r"
            <?php

            $obj?->method($x = 5);
        "}
    }

    test_lint_failure! {
        name = assignment_with_parentheses,
        rule = NoAssignInArgumentRule,
        count = 1,
        code = indoc! {r"
            <?php

            foo(($x = 5));
        "}
    }

    test_lint_failure! {
        name = multiple_arguments_with_assignment,
        rule = NoAssignInArgumentRule,
        count = 1,
        code = indoc! {r"
            <?php

            foo($a, $b = 5, $c);
        "}
    }

    test_lint_success! {
        name = variable_used_as_argument,
        rule = NoAssignInArgumentRule,
        code = indoc! {r"
            <?php

            $x = 5;
            foo($x);
        "}
    }

    test_lint_success! {
        name = expression_without_assignment,
        rule = NoAssignInArgumentRule,
        code = indoc! {r"
            <?php

            foo($x + 5);
        "}
    }

    test_lint_success! {
        name = function_call_as_argument,
        rule = NoAssignInArgumentRule,
        code = indoc! {r"
            <?php

            foo(getValue());
        "}
    }
}
