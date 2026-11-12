use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::Argument;
use mago_syntax::cst::Call;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Literal;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches_any;
use crate::rule::utils::consts::NO_NAMED_ARGUMENTS_FUNCTIONS;
use crate::rule::utils::consts::VARIADIC_FUNCTIONS;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct LiteralNamedArgumentRule {
    meta: &'static RuleMeta,
    cfg: LiteralNamedArgumentConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct LiteralNamedArgumentConfig {
    pub level: Level,
    pub check_first_argument: bool,
    /// Minimum number of literal positional arguments in a single call before reporting.
    ///
    /// Default is `1`, meaning any literal positional argument triggers a warning.
    /// Set higher (e.g., `3`) to only flag calls with many unnamed literals.
    pub threshold: usize,
}

impl Default for LiteralNamedArgumentConfig {
    fn default() -> Self {
        Self { level: Level::Warning, check_first_argument: false, threshold: 1 }
    }
}

impl Config for LiteralNamedArgumentConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for LiteralNamedArgumentRule {
    type Config = LiteralNamedArgumentConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Literal Named Argument",
            code: "literal-named-argument",
            description: indoc! {r"
                Enforces that literal values used as arguments in function or method calls
                are passed as **named arguments**.

                This improves readability by clarifying the purpose of the literal value at the call site.
                It is particularly helpful for boolean flags, numeric constants, and `null` values
                where the intent is often ambiguous without the parameter name.
            "},
            good_example: indoc! {r"
                <?php

                function set_option(string $key, bool $enable_feature) {}

                set_option(key: 'feature_x', enable_feature: true); // ✅ clear intent
            "},
            bad_example: indoc! {r"
                <?php

                function set_option(string $key, bool $enable_feature) {}

                set_option('feature_x', true); // ❌ intent unclear
            "},
            category: Category::Clarity,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP80)),
        };
        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Call];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Call(call) = node else {
            return;
        };

        // Skip variadic functions and functions marked with @no-named-arguments
        if let Call::Function(function_call) = call
            && (function_call_matches_any(ctx, function_call, &VARIADIC_FUNCTIONS).is_some()
                || function_call_matches_any(ctx, function_call, &NO_NAMED_ARGUMENTS_FUNCTIONS).is_some())
        {
            return;
        }

        let mut literal_args: Vec<(&Literal<'arena>, &'arena [u8])> = Vec::new();

        for (index, argument) in call.get_argument_list().arguments.iter().enumerate() {
            if index == 0 && !self.cfg.check_first_argument {
                continue;
            }

            let Argument::Positional(positional_argument) = argument else {
                continue;
            };

            let Expression::Literal(literal) = &positional_argument.value else {
                continue;
            };

            let literal_value: &[u8] = match literal {
                Literal::String(lit_str) => lit_str.raw,
                Literal::Integer(lit_int) => lit_int.raw,
                Literal::Float(lit_float) => lit_float.raw,
                Literal::True(_) => b"true",
                Literal::False(_) => b"false",
                Literal::Null(_) => b"null",
            };

            literal_args.push((literal, literal_value));
        }

        if literal_args.len() < self.cfg.threshold {
            return;
        }

        for (literal, literal_value) in literal_args {
            let literal_display = mago_bytes::BytesDisplay(literal_value);
            ctx.collector.report(
                Issue::new(
                    self.cfg.level,
                    format!(
                        "Literal argument `{literal_display}` should be passed as a named argument for clarity."
                    ),
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(literal.span()).with_message("This literal is being passed positionally."),
                )
                .with_note(
                    "Passing literals positionally can make code less clear, especially with booleans, numbers, or `null`.",
                )
                .with_help(format!("Consider using a named argument instead: `function_name(param: {literal_display})`.")),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;
    use indoc::indoc;

    test_lint_success! {
        name = first_argument_skipped_by_default,
        rule = LiteralNamedArgumentRule,
        code = indoc! {r"
            <?php

            config('app.name');
        "}
    }

    test_lint_failure! {
        name = second_argument_checked,
        rule = LiteralNamedArgumentRule,
        count = 1,
        code = indoc! {r"
            <?php

            data_set($array, 'key');
        "}
    }

    test_lint_failure! {
        name = first_argument_checked_when_enabled,
        rule = LiteralNamedArgumentRule,
        settings = |s: &mut crate::settings::Settings| s.rules.literal_named_argument.config.check_first_argument = true,
        code = indoc! {r"
            <?php

            config('app.name');
        "}
    }

    test_lint_success! {
        name = named_arguments_are_fine,
        rule = LiteralNamedArgumentRule,
        code = indoc! {r"
            <?php

            set_option(key: 'foo', value: true);
        "}
    }

    test_lint_success! {
        name = below_threshold_no_report,
        rule = LiteralNamedArgumentRule,
        settings = |s: &mut crate::settings::Settings| s.rules.literal_named_argument.config.threshold = 3,
        code = indoc! {r"
            <?php

            foo('a', 'b');
        "}
    }

    test_lint_failure! {
        name = at_threshold_reports,
        rule = LiteralNamedArgumentRule,
        count = 2,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.literal_named_argument.config.threshold = 2;
            s.rules.literal_named_argument.config.check_first_argument = true;
        },
        code = indoc! {r"
            <?php

            foo('a', 'b');
        "}
    }

    test_lint_failure! {
        name = above_threshold_reports_all,
        rule = LiteralNamedArgumentRule,
        count = 3,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.literal_named_argument.config.threshold = 2;
            s.rules.literal_named_argument.config.check_first_argument = true;
        },
        code = indoc! {r"
            <?php

            foo('a', 'b', 'c');
        "}
    }

    test_lint_success! {
        name = threshold_with_first_arg_skipped,
        rule = LiteralNamedArgumentRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.literal_named_argument.config.threshold = 2;
            s.rules.literal_named_argument.config.check_first_argument = false;
        },
        code = indoc! {r"
            <?php

            foo('a', 'b');
        "}
    }

    test_lint_failure! {
        name = instance_method_arguments_checked,
        rule = LiteralNamedArgumentRule,
        count = 1,
        code = indoc! {r#"
            <?php

            $this->curlPost("/some_endpoint/", $data, 57);
        "#}
    }

    test_lint_failure! {
        name = static_method_arguments_checked,
        rule = LiteralNamedArgumentRule,
        count = 1,
        code = indoc! {r"
            <?php

            Client::post($endpoint, $data, 57);
        "}
    }

    test_lint_failure! {
        name = nullsafe_method_arguments_checked,
        rule = LiteralNamedArgumentRule,
        count = 1,
        code = indoc! {r"
            <?php

            $client?->post($endpoint, $data, 57);
        "}
    }

    test_lint_failure! {
        name = temporary_object_method_arguments_checked,
        rule = LiteralNamedArgumentRule,
        count = 1,
        code = indoc! {r"
            <?php

            (new Client())->post($endpoint, $data, 57);
        "}
    }

    test_lint_failure! {
        name = method_first_argument_checked_when_enabled,
        rule = LiteralNamedArgumentRule,
        count = 4,
        settings = |s: &mut crate::settings::Settings| s.rules.literal_named_argument.config.check_first_argument = true,
        code = indoc! {r"
            <?php

            $client->configure(true);
            Client::configure(57);
            $client?->configure(null);
            (new Client())->configure('value');
        "}
    }

    test_lint_success! {
        name = method_first_argument_skipped_by_default,
        rule = LiteralNamedArgumentRule,
        code = indoc! {r"
            <?php

            $client->configure(true);
            Client::configure(57);
            $client?->configure(null);
            (new Client())->configure('value');
        "}
    }

    test_lint_success! {
        name = named_method_arguments_are_fine,
        rule = LiteralNamedArgumentRule,
        code = indoc! {r"
            <?php

            $client->post($endpoint, $data, timeout: 57);
            Client::post($endpoint, $data, timeout: 57);
            $client?->post($endpoint, $data, timeout: 57);
            (new Client())->post($endpoint, $data, timeout: 57);
        "}
    }

    test_lint_success! {
        name = method_arguments_below_threshold,
        rule = LiteralNamedArgumentRule,
        settings = |s: &mut crate::settings::Settings| s.rules.literal_named_argument.config.threshold = 3,
        code = indoc! {r"
            <?php

            $client->configure($key, true, 57);
            Client::configure($key, true, 57);
            $client?->configure($key, true, 57);
            (new Client())->configure($key, true, 57);
        "}
    }

    test_lint_failure! {
        name = method_arguments_at_threshold,
        rule = LiteralNamedArgumentRule,
        count = 8,
        settings = |s: &mut crate::settings::Settings| s.rules.literal_named_argument.config.threshold = 2,
        code = indoc! {r"
            <?php

            $client->configure($key, true, 57);
            Client::configure($key, true, 57);
            $client?->configure($key, true, 57);
            (new Client())->configure($key, true, 57);
        "}
    }

    test_lint_success! {
        name = non_literal_method_arguments_are_fine,
        rule = LiteralNamedArgumentRule,
        code = indoc! {r"
            <?php

            $client->post($endpoint, $data, $timeout);
            Client::post($endpoint, $data, $timeout);
            $client?->post($endpoint, $data, $timeout);
            (new Client())->post($endpoint, $data, $timeout);
        "}
    }

    test_lint_failure! {
        name = dynamic_method_arguments_checked,
        rule = LiteralNamedArgumentRule,
        count = 3,
        code = indoc! {r"
            <?php

            $client->{$method}($key, true);
            Client::{$method}($key, true);
            $client?->{$method}($key, true);
        "}
    }

    test_lint_failure! {
        name = methods_do_not_inherit_function_exclusions,
        rule = LiteralNamedArgumentRule,
        count = 3,
        code = indoc! {r"
            <?php

            $client->sprintf($format, 57);
            Client::strcmp($value, 'expected');
            $client?->var_dump($value, true);
        "}
    }

    test_lint_success! {
        name = excluded_functions_are_still_skipped,
        rule = LiteralNamedArgumentRule,
        code = indoc! {r"
            <?php

            sprintf('%d', 57);
            strcmp($value, 'expected');
        "}
    }
}
