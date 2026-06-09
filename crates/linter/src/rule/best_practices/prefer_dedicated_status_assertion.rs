use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Argument;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Literal;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferDedicatedStatusAssertionRule {
    meta: &'static RuleMeta,
    cfg: PreferDedicatedStatusAssertionConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct PreferDedicatedStatusAssertionConfig {
    pub level: Level,
}

impl Default for PreferDedicatedStatusAssertionConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for PreferDedicatedStatusAssertionConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferDedicatedStatusAssertionRule {
    type Config = PreferDedicatedStatusAssertionConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Dedicated Status Assertion",
            code: "prefer-dedicated-status-assertion",
            description: indoc! {"
                Detects calls to `assertStatus()` with a status code that has a dedicated assertion
                method, such as `assertOk()` for `200` or `assertNotFound()` for `404`. The dedicated
                methods read better and state the expected outcome directly.
            "},
            good_example: indoc! {r"
                <?php

                $response->assertOk();
            "},
            bad_example: indoc! {r"
                <?php

                $response->assertStatus(200);
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::Integration(Integration::Laravel),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::MethodCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::MethodCall(method_call) = node else {
            return;
        };

        let ClassLikeMemberSelector::Identifier(method_name) = &method_call.method else {
            return;
        };

        if !method_name.value.eq_ignore_ascii_case(b"assertStatus") {
            return;
        }

        let arguments = &method_call.argument_list.arguments;
        if arguments.len() != 1 {
            return;
        }

        let Some(Argument::Positional(argument)) = arguments.first() else {
            return;
        };

        if argument.ellipsis.is_some() {
            return;
        }

        let Expression::Literal(Literal::Integer(integer)) = argument.value else {
            return;
        };

        let Some(status_code) = integer.value else {
            return;
        };

        let Some(method) = dedicated_status_method(status_code) else {
            return;
        };

        let span = method_name.span().join(method_call.argument_list.right_parenthesis);

        let issue = Issue::new(
            self.cfg.level(),
            format!("Use the dedicated `{method}()` assertion instead of `assertStatus({status_code})`."),
        )
        .with_code(self.meta.code)
        .with_annotation(Annotation::primary(span).with_message(format!("This can be written as `{method}()`")))
        .with_note(format!("`{method}()` is the dedicated assertion for the `{status_code}` status code."))
        .with_help(format!("Replace `assertStatus({status_code})` with `{method}()`."));

        ctx.collector.propose(issue, |edits| {
            edits.push(TextEdit::replace(span, format!("{method}()")));
        });
    }
}

/// Maps an HTTP status code to Laravel's dedicated test-response assertion, if one exists.
///
/// Mirrors `Illuminate\Testing\Concerns\AssertsStatusCodes`.
const fn dedicated_status_method(status_code: u64) -> Option<&'static str> {
    Some(match status_code {
        200 => "assertOk",
        201 => "assertCreated",
        202 => "assertAccepted",
        204 => "assertNoContent",
        301 => "assertMovedPermanently",
        302 => "assertFound",
        304 => "assertNotModified",
        307 => "assertTemporaryRedirect",
        308 => "assertPermanentRedirect",
        400 => "assertBadRequest",
        401 => "assertUnauthorized",
        402 => "assertPaymentRequired",
        403 => "assertForbidden",
        404 => "assertNotFound",
        405 => "assertMethodNotAllowed",
        406 => "assertNotAcceptable",
        408 => "assertRequestTimeout",
        409 => "assertConflict",
        410 => "assertGone",
        415 => "assertUnsupportedMediaType",
        422 => "assertUnprocessable",
        424 => "assertFailedDependency",
        429 => "assertTooManyRequests",
        500 => "assertInternalServerError",
        503 => "assertServiceUnavailable",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_failure! {
        name = assert_status_ok,
        rule = PreferDedicatedStatusAssertionRule,
        code = indoc! {r"
            <?php

            $response->assertStatus(200);
        "}
    }

    test_lint_failure! {
        name = assert_status_not_found,
        rule = PreferDedicatedStatusAssertionRule,
        code = indoc! {r"
            <?php

            $this->get('/missing')->assertStatus(404);
        "}
    }

    test_lint_success! {
        name = dedicated_method_used,
        rule = PreferDedicatedStatusAssertionRule,
        code = indoc! {r"
            <?php

            $response->assertOk();
        "}
    }

    test_lint_success! {
        name = unknown_status_code,
        rule = PreferDedicatedStatusAssertionRule,
        code = indoc! {r"
            <?php

            $response->assertStatus(599);
        "}
    }

    test_lint_success! {
        name = non_literal_status_code,
        rule = PreferDedicatedStatusAssertionRule,
        code = indoc! {r"
            <?php

            $response->assertStatus($code);
        "}
    }

    test_lint_fix! {
        name = fix_assert_status_to_not_found,
        rule = PreferDedicatedStatusAssertionRule,
        code = indoc! {r"
            <?php

            $response->assertStatus(404);
        "},
        fixed = indoc! {r"
            <?php

            $response->assertNotFound();
        "}
    }
}
