use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::Call;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Literal;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Variable;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const IS_OBJECT: &str = "is_object";
const METHOD_EXISTS: &str = "method_exists";
const TO_STRING_METHOD: &str = "__toString";

#[derive(Debug, Clone)]
pub struct InstanceofStringableRule {
    meta: &'static RuleMeta,
    cfg: InstanceofStringableConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct InstanceofStringableConfig {
    pub level: Level,
}

impl Default for InstanceofStringableConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for InstanceofStringableConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for InstanceofStringableRule {
    type Config = InstanceofStringableConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Instanceof Stringable",
            code: "instanceof-stringable",
            description: indoc! {"
                Detects the legacy pattern `is_object($x) && method_exists($x, '__toString')` and suggests
                replacing it with `$x instanceof Stringable` for improved readability and performance.

                Since PHP 8.0, all classes with `__toString()` automatically implement the `Stringable` interface.
            "},
            good_example: indoc! {r"
                <?php

                function stringify(mixed $value): string {
                    if ($value instanceof Stringable) {
                        return (string) $value;
                    }

                    return '';
                }
            "},
            bad_example: indoc! {r"
                <?php

                function stringify(mixed $value): string {
                    if (is_object($value) && method_exists($value, '__toString')) {
                        return (string) $value;
                    }

                    return '';
                }
            "},
            category: Category::Clarity,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP80)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Binary];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Binary(binary) = node else { return };

        if !matches!(binary.operator, BinaryOperator::And(_)) {
            return;
        }

        let (is_object_call, method_exists_call) = match (binary.lhs, binary.rhs) {
            (Expression::Call(Call::Function(left_call)), Expression::Call(Call::Function(right_call))) => {
                if function_call_matches(ctx, left_call, IS_OBJECT)
                    && function_call_matches(ctx, right_call, METHOD_EXISTS)
                {
                    (left_call, right_call)
                } else if function_call_matches(ctx, left_call, METHOD_EXISTS)
                    && function_call_matches(ctx, right_call, IS_OBJECT)
                {
                    (right_call, left_call)
                } else {
                    return;
                }
            }
            _ => return,
        };

        if is_object_call.argument_list.arguments.len() != 1 {
            return;
        }

        if method_exists_call.argument_list.arguments.len() != 2 {
            return;
        }

        let Some(is_object_arg) = is_object_call.argument_list.arguments.first() else {
            return;
        };

        let Expression::Variable(Variable::Direct(is_object_var)) = is_object_arg.value() else {
            return;
        };

        let Some(method_exists_first_arg) = method_exists_call.argument_list.arguments.first() else {
            return;
        };

        let Expression::Variable(Variable::Direct(method_exists_var)) = method_exists_first_arg.value() else {
            return;
        };

        if is_object_var.name != method_exists_var.name {
            return;
        }

        let Some(arg) = method_exists_call.argument_list.arguments.get(1) else {
            return;
        };

        let Expression::Literal(Literal::String(string_lit)) = arg.value() else {
            return;
        };

        let Some(method_name) = string_lit.value else {
            return;
        };

        if !method_name.eq_ignore_ascii_case(TO_STRING_METHOD) {
            return;
        }

        ctx.collector.report(
            Issue::new(
                self.cfg.level,
                "Consider using `instanceof Stringable` instead of `is_object() && method_exists()`.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(binary.span())
                    .with_message("This pattern can be replaced with `instanceof Stringable`"),
            )
            .with_note("Since PHP 8.0, all classes with `__toString()` automatically implement `Stringable`.")
            .with_help("`instanceof Stringable` is clearer and more performant."),
        );
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::InstanceofStringableRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = instanceof_stringable_is_preferred,
        rule = InstanceofStringableRule,
        code = indoc! {r"
            <?php

            function stringify(mixed $value): string {
                if ($value instanceof Stringable) {
                    return (string) $value;
                }

                return '';
            }
        "}
    }

    test_lint_failure! {
        name = is_object_and_method_exists_detected,
        rule = InstanceofStringableRule,
        code = indoc! {r"
            <?php

            if (is_object($x) && method_exists($x, '__toString')) {
                echo (string) $x;
            }
        "}
    }

    test_lint_failure! {
        name = reversed_order_detected,
        rule = InstanceofStringableRule,
        code = indoc! {r"
            <?php

            if (method_exists($x, '__toString') && is_object($x)) {
                echo (string) $x;
            }
        "}
    }

    test_lint_failure! {
        name = case_insensitive_method_name_uppercase,
        rule = InstanceofStringableRule,
        code = indoc! {r"
            <?php

            if (is_object($x) && method_exists($x, '__TOSTRING')) {
                echo (string) $x;
            }
        "}
    }

    test_lint_failure! {
        name = case_insensitive_method_name_lowercase,
        rule = InstanceofStringableRule,
        code = indoc! {r"
            <?php

            if (is_object($x) && method_exists($x, '__tostring')) {
                echo (string) $x;
            }
        "}
    }

    test_lint_success! {
        name = different_method_name_not_flagged,
        rule = InstanceofStringableRule,
        code = indoc! {r"
            <?php

            if (is_object($x) && method_exists($x, 'serialize')) {
                echo $x->serialize();
            }
        "}
    }

    test_lint_success! {
        name = different_variables_not_flagged,
        rule = InstanceofStringableRule,
        code = indoc! {r"
            <?php

            if (is_object($x) && method_exists($y, '__toString')) {
                echo (string) $y;
            }
        "}
    }
}
