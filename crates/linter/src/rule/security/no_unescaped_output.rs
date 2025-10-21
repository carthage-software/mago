use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoUnescapedOutputRule {
    meta: &'static RuleMeta,
    cfg: NoUnescapedOutputConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
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
            bad_example: indoc! {r#"
                <?php

                // This is a major XSS vulnerability.
                echo $_GET['user_comment'];
            "#},
            category: Category::Security,
            requirements: RuleRequirements::Integration(Integration::WordPress),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Echo, NodeKind::PrintConstruct, NodeKind::FunctionCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        match node {
            Node::Echo(echo) => {
                // Check each expression in the echo statement
                for expression in echo.values.iter() {
                    if needs_escaping_with_context(expression, Some(ctx)) {
                        self.report_unescaped_output(ctx, expression.span(), "echo statement");
                    }
                }
            }
            Node::EchoTag(echo_tag) => {
                // Check each expression in the echo statement
                for expression in echo_tag.values.iter() {
                    if needs_escaping_with_context(expression, Some(ctx)) {
                        self.report_unescaped_output(ctx, expression.span(), "echo tag");
                    }
                }
            }
            Node::PrintConstruct(print_construct) => {
                // Check the print construct expression
                if needs_escaping_with_context(print_construct.value, Some(ctx)) {
                    self.report_unescaped_output(ctx, print_construct.value.span(), "print statement");
                }
            }
            Node::FunctionCall(function_call) => {
                // Check printf function - only flag if it has exactly one argument (the format string)
                if function_call.argument_list.arguments.len() == 1
                    && function_call_matches(ctx, function_call, "printf")
                    && let Some(first_arg) = function_call.argument_list.arguments.first().map(|arg| arg.value())
                    && needs_escaping_with_context(first_arg, Some(ctx))
                {
                    self.report_unescaped_output(ctx, first_arg.span(), "printf function");
                }
            }
            _ => {}
        }
    }
}

impl NoUnescapedOutputRule {
    fn report_unescaped_output<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, span: mago_span::Span, context: &str) {
        let issue = Issue::new(self.cfg.level(), "All output should be escaped to prevent XSS vulnerabilities")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(span).with_message(format!("Unescaped output in {}", context)))
            .with_note("Unescaped data can lead to Cross-Site Scripting vulnerabilities")
            .with_help("Use `esc_html()`, `esc_attr()`, `esc_url()`, etc.");

        ctx.collector.report(issue);
    }
}

/// Check if an expression needs escaping before output (with context)
fn needs_escaping_with_context(expr: &Expression, ctx: Option<&LintContext>) -> bool {
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
            if let Some(context) = ctx {
                !is_escaping_function_call(context, function_call)
            } else {
                // Fallback: if no context, check by identifier value
                if let Expression::Identifier(function_name) = function_call.function {
                    !is_escaping_function(function_name.value())
                } else {
                    true
                }
            }
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

/// Check if a function call is a WordPress escaping function
fn is_escaping_function_call(ctx: &LintContext, function_call: &FunctionCall) -> bool {
    let escaping_functions = [
        "esc_html",
        "esc_attr",
        "esc_url",
        "esc_js",
        "esc_textarea",
        "esc_xml",
        "sanitize_text_field",
        "sanitize_email",
        "sanitize_url",
        "wp_kses",
        "wp_kses_post",
    ];

    for func_name in escaping_functions {
        if function_call_matches(ctx, function_call, func_name) {
            return true;
        }
    }

    false
}

/// Check if a function name is a WordPress escaping function (fallback without context)
fn is_escaping_function(name: &str) -> bool {
    matches!(
        name,
        "esc_html"
            | "esc_attr"
            | "esc_url"
            | "esc_js"
            | "esc_textarea"
            | "esc_xml"
            | "sanitize_text_field"
            | "sanitize_email"
            | "sanitize_url"
            | "wp_kses"
            | "wp_kses_post"
    )
}
