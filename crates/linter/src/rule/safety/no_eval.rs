use indoc::indoc;
use schemars::JsonSchema;
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
pub struct NoEvalRule {
    meta: &'static RuleMeta,
    cfg: NoEvalConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoEvalConfig {
    pub level: Level,
}

impl Default for NoEvalConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoEvalConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoEvalRule {
    type Config = NoEvalConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Eval",
            code: "no-eval",
            description: indoc! {"
                Detects unsafe uses of the `eval` construct.
                The `eval` construct executes arbitrary code, which can be a major security risk if not used carefully.
            "},
            good_example: indoc! {r"
                <?php

                // Safe alternative to eval
                $result = json_decode($jsonString);
            "},
            bad_example: indoc! {r#"
                <?php

                eval('echo "Hello, world!";');
            "#},
            category: Category::Safety,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::EvalConstruct];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::EvalConstruct(eval_construct) = node else {
            return;
        };

        ctx.collector.report(
            Issue::new(self.cfg.level(), "Unsafe use of `eval` construct.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(eval_construct.eval.span)
                        .with_message("This `eval` construct is unsafe"),
                )
                .with_annotation(
                    Annotation::secondary(eval_construct.value.span())
                        .with_message("The evaluated code is here"),
                )
                .with_note("The `eval` construct executes arbitrary code, which can be a major security risk if not used carefully.")
                .with_note("It can potentially lead to remote code execution vulnerabilities if the evaluated code is not properly sanitized.")
                .with_note("Consider using safer alternatives whenever possible.")
                .with_help("Avoid using `eval` unless absolutely necessary, and ensure that any dynamically generated code is properly validated and sanitized before execution."),
        );
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoEvalRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = json_decode_is_safe,
        rule = NoEvalRule,
        code = indoc! {r"
            <?php

            $result = json_decode($jsonString);
            echo $result;
        "}
    }

    test_lint_success! {
        name = regular_function_call_is_safe,
        rule = NoEvalRule,
        code = indoc! {r#"
            <?php

            function process($data) {
                return strtoupper($data);
            }

            $output = process("hello");
        "#}
    }

    test_lint_success! {
        name = string_eval_variable_name_is_not_eval_construct,
        rule = NoEvalRule,
        code = indoc! {r#"
            <?php

            $eval = "some value";
            echo $eval;
        "#}
    }

    test_lint_failure! {
        name = direct_eval_call_fails,
        rule = NoEvalRule,
        code = indoc! {r#"
            <?php

            eval('echo "Hello, world!";');
        "#}
    }

    test_lint_failure! {
        name = eval_with_variable_fails,
        rule = NoEvalRule,
        code = indoc! {r"
            <?php

            $code = '$a = 1 + 2;';
            eval($code);
        "}
    }

    test_lint_failure! {
        name = eval_in_condition_fails,
        rule = NoEvalRule,
        code = indoc! {r"
            <?php

            if ($needsEval) {
                eval($userInput);
            }
        "}
    }

    test_lint_failure! {
        name = multiple_evals_detected,
        rule = NoEvalRule,
        count = 3,
        code = indoc! {r"
            <?php

            eval($code1);
            eval($code2);
            eval($code3);
        "}
    }

    test_lint_failure! {
        name = nested_eval_counted,
        rule = NoEvalRule,
        count = 2,
        code = indoc! {r"
            <?php

            function dangerous() {
                eval('$x = 1;');

                if (true) {
                    eval('$y = 2;');
                }
            }
        "}
    }
}
