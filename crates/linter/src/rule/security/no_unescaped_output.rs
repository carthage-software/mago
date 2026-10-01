use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::Call;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Literal;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches;
use crate::rule::utils::call::function_call_matches_any;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoUnescapedOutputRule {
    meta: &'static RuleMeta,
    cfg: NoUnescapedOutputConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoUnescapedOutputConfig {
    pub level: Level,
}

impl Default for NoUnescapedOutputConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoUnescapedOutputConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoUnescapedOutputRule {
    type Config = NoUnescapedOutputConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Unescaped Output",
            code: "no-unescaped-output",
            description: indoc! {"
                This rule ensures that any variable or function call that is output directly to the page is
                properly escaped. All data must be escaped before printing to prevent Cross-Site Scripting (XSS)
                vulnerabilities.
            "},
            good_example: indoc! {r#"
                <?php

                echo esc_html( $user_comment );
                ?>
                <a href="<?php echo esc_url( $user_provided_url ); ?>">Link</a>
            "#},
            bad_example: indoc! {r"
                <?php

                // This is a major XSS vulnerability.
                echo $_GET['user_comment'];
            "},
            category: Category::Security,
            requirements: RuleRequirements::Integration(Integration::WordPress),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] =
            &[NodeKind::Echo, NodeKind::EchoTag, NodeKind::PrintConstruct, NodeKind::FunctionCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        match node {
            Node::Echo(echo) => {
                // Check each expression in the echo statement
                for expression in &echo.values {
                    if needs_escaping_with_context(expression, ctx) {
                        self.report_unescaped_output(ctx, expression.span(), "echo statement");
                    }
                }
            }
            Node::EchoTag(echo_tag) => {
                // Check each expression in the echo statement
                for expression in &echo_tag.values {
                    if needs_escaping_with_context(expression, ctx) {
                        self.report_unescaped_output(ctx, expression.span(), "echo tag");
                    }
                }
            }
            // Check the print construct expression
            Node::PrintConstruct(print_construct) if needs_escaping_with_context(print_construct.value, ctx) => {
                self.report_unescaped_output(ctx, print_construct.value.span(), "print statement");
            }
            Node::FunctionCall(function_call) => {
                // Check printf function - only flag if it has exactly one argument (the format string)
                if function_call.argument_list.arguments.len() == 1
                    && function_call_matches(ctx, function_call, "printf")
                    && let Some(first_arg) =
                        function_call.argument_list.arguments.first().map(mago_syntax::cst::Argument::value)
                    && needs_escaping_with_context(first_arg, ctx)
                {
                    self.report_unescaped_output(ctx, first_arg.span(), "printf function");
                }
            }
            _ => {}
        }
    }
}

impl NoUnescapedOutputRule {
    fn report_unescaped_output<A>(&self, ctx: &mut LintContext<'_, '_, A>, span: mago_span::Span, context: &str)
    where
        A: Arena,
    {
        let issue = Issue::new(self.cfg.level(), "All output should be escaped to prevent XSS vulnerabilities")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(span).with_message(format!("Unescaped output in {context}")))
            .with_note("Unescaped data can lead to Cross-Site Scripting vulnerabilities")
            .with_help("Use `esc_html()`, `esc_attr()`, `esc_url()`, etc.");

        ctx.collector.report(issue);
    }
}

/// Check if an expression needs escaping before output (with context)
fn needs_escaping_with_context<A>(expr: &Expression, ctx: &LintContext<'_, '_, A>) -> bool
where
    A: Arena,
{
    match expr {
        // Literal strings and numbers are generally safe
        Expression::Literal(Literal::String(_)) => false,
        Expression::Literal(Literal::Integer(_)) => false,
        Expression::Literal(Literal::Float(_)) => false,
        // Variables are potentially unsafe
        Expression::Variable(_) => true,
        // Array access is potentially unsafe
        Expression::ArrayAccess(_) => true,
        // Function calls - check if it's already an escaping function
        Expression::Call(Call::Function(function_call)) => {
            function_call_matches_any(ctx, function_call, SAFE_OUTPUT_FUNCTIONS).is_none()
        }
        // Method calls and property access are potentially unsafe
        Expression::Call(_) => true,
        Expression::Access(_) => true,
        // Binary operations might be unsafe
        Expression::Binary(binary) => {
            needs_escaping_with_context(binary.lhs, ctx) || needs_escaping_with_context(binary.rhs, ctx)
        }
        // Conditional expressions might be unsafe
        Expression::Conditional(conditional) => {
            (if let Some(then_expr) = conditional.then { needs_escaping_with_context(then_expr, ctx) } else { false })
                || needs_escaping_with_context(conditional.r#else, ctx)
        }
        // Other expressions are potentially unsafe
        _ => true,
    }
}

/// `WordPress` functions whose return value is escaped or sanitized for HTML output.
///
/// Mirrors the `escapingFunctions` list of WPCS `EscapingFunctionsTrait`, minus functions that
/// escape for a non-HTML context (`esc_sql`, `like_escape`, `esc_url_raw`) or only read input
/// (`filter_input`, `filter_var`).
const SAFE_OUTPUT_FUNCTIONS: &[&str] = &[
    "absint",
    "esc_attr",
    "esc_attr__",
    "esc_attr_e",
    "esc_attr_x",
    "esc_html",
    "esc_html__",
    "esc_html_e",
    "esc_html_x",
    "esc_js",
    "esc_textarea",
    "esc_url",
    "esc_xml",
    "floatval",
    "highlight_string",
    "intval",
    "json_encode",
    "number_format",
    "rawurlencode",
    "sanitize_email",
    "sanitize_hex_color",
    "sanitize_hex_color_no_hash",
    "sanitize_html_class",
    "sanitize_key",
    "sanitize_locale_name",
    "sanitize_text_field",
    "sanitize_url",
    "sanitize_user_field",
    "tag_escape",
    "urlencode",
    "urlencode_deep",
    "wp_json_encode",
    "wp_kses",
    "wp_kses_data",
    "wp_kses_one_attr",
    "wp_kses_post",
];

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoUnescapedOutputRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = translated_and_escaped_output_is_safe,
        rule = NoUnescapedOutputRule,
        code = indoc! {r#"
            <?php

            echo esc_html__( 'Hello', 'my-plugin' );
            echo wp_json_encode( $data );
            echo absint( $count );
            ?>
            <span><?= esc_attr( $title ) ?></span>
        "#}
    }

    test_lint_success! {
        name = namespaced_escaping_call_is_safe,
        rule = NoUnescapedOutputRule,
        code = indoc! {r"
            <?php

            namespace App;

            echo Esc_Html( $title );
        "}
    }

    test_lint_failure! {
        name = echo_tag_output_is_checked,
        rule = NoUnescapedOutputRule,
        code = indoc! {r"
            <?php $title = $_GET['title']; ?>
            <h1><?= $title ?></h1>
        "}
    }

    test_lint_failure! {
        name = non_html_escaping_is_not_safe,
        rule = NoUnescapedOutputRule,
        count = 3,
        code = indoc! {r"
            <?php

            echo esc_sql( $_GET['q'] );
            echo esc_url_raw( $url );
            echo filter_input( INPUT_GET, 'q' );
        "}
    }
}
