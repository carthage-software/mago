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
use mago_syntax::cst::BinaryOperator;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Literal;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferTimestampFactoryRule {
    meta: &'static RuleMeta,
    cfg: PreferTimestampFactoryConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct PreferTimestampFactoryConfig {
    pub level: Level,
}

impl Default for PreferTimestampFactoryConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for PreferTimestampFactoryConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferTimestampFactoryRule {
    type Config = PreferTimestampFactoryConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer DateTimeImmutable Create From Timestamp",
            code: "prefer-datetimeimmutable-create-from-timestamp",
            description: indoc! {r"
                Suggests `DateTimeImmutable::createFromTimestamp()` when a Unix timestamp is converted
                to a string and parsed by the `DateTimeImmutable` constructor.

                The factory accepts an `int` or `float` timestamp directly, avoiding the intermediate
                string construction and date parsing step.
            "},
            good_example: indoc! {r"
                <?php

                $timestamp = time();
                $date = DateTimeImmutable::createFromTimestamp($timestamp);
            "},
            bad_example: indoc! {r"
                <?php

                $timestamp = time();
                $date = new DateTimeImmutable('@' . $timestamp);
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP84)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Instantiation];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Instantiation(instantiation) = node else {
            return;
        };

        let Expression::Identifier(identifier) = instantiation.class else {
            return;
        };

        if !ctx.lookup_name(identifier).eq_ignore_ascii_case(b"datetimeimmutable") {
            return;
        }

        let Some(argument_list) = &instantiation.argument_list else {
            return;
        };

        if argument_list.arguments.len() != 1 {
            return;
        }

        let Some(Argument::Positional(argument)) = argument_list.arguments.first() else {
            return;
        };

        if argument.ellipsis.is_some() {
            return;
        }

        let Expression::Binary(binary) = argument.value else {
            return;
        };

        if !matches!(binary.operator, BinaryOperator::StringConcat(_)) {
            return;
        }

        let Expression::Literal(Literal::String(prefix)) = binary.lhs else {
            return;
        };

        if prefix.value != Some(b"@".as_slice()) {
            return;
        }

        ctx.collector.report(
            Issue::new(
                self.cfg.level(),
                "Use `DateTimeImmutable::createFromTimestamp()` instead of parsing a timestamp string.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(instantiation.span())
                    .with_message("This constructs and parses a timestamp string."),
            )
            .with_note("`DateTimeImmutable::createFromTimestamp()` accepts Unix timestamps directly.")
            .with_help("Use `DateTimeImmutable::createFromTimestamp($timestamp)` instead."),
        );
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::PreferTimestampFactoryRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = timestamp_string_constructor,
        rule = PreferTimestampFactoryRule,
        code = indoc! {r"
            <?php

            $timestamp = time();
            $date = new DateTimeImmutable('@' . $timestamp);
        "}
    }

    test_lint_failure! {
        name = fully_qualified_timestamp_string_constructor,
        rule = PreferTimestampFactoryRule,
        code = indoc! {r"
            <?php

            $date = new \DateTimeImmutable('@' . time());
        "}
    }

    test_lint_failure! {
        name = aliased_timestamp_string_constructor,
        rule = PreferTimestampFactoryRule,
        code = indoc! {r"
            <?php

            namespace App;

            use DateTimeImmutable as ImmutableDateTime;

            $date = new ImmutableDateTime('@' . $timestamp);
        "}
    }

    test_lint_success! {
        name = other_datetime_constructors_are_not_reported,
        rule = PreferTimestampFactoryRule,
        code = indoc! {r"
            <?php

            $timestamp = time();

            $already_direct = DateTimeImmutable::createFromTimestamp($timestamp);
            $non_timestamp = new DateTimeImmutable($timestamp);
            $with_timezone = new DateTimeImmutable('@' . $timestamp, new DateTimeZone('UTC'));
            $mutable = new DateTime('@' . $timestamp);
            $not_a_timestamp = new DateTimeImmutable('timestamp: ' . $timestamp);
        "}
    }

    test_lint_success! {
        name = namespaced_class_with_the_same_short_name_is_not_reported,
        rule = PreferTimestampFactoryRule,
        code = indoc! {r"
            <?php

            namespace App;

            class DateTimeImmutable {}

            $date = new DateTimeImmutable('@' . $timestamp);
        "}
    }
}
