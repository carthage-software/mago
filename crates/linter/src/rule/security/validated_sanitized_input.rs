use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Argument;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::Call;
use mago_syntax::ast::Expression;
use mago_syntax::ast::MethodBody;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Statement;
use mago_syntax::ast::UnaryPrefixOperator;
use mago_syntax::ast::Variable;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches_any;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const INPUT_SUPERGLOBALS: &[&str] = &["$_GET", "$_POST", "$_REQUEST", "$_COOKIE", "$_SERVER"];

const SANITIZATION_FUNCTIONS: &[&str] = &[
    "_wp_handle_upload",
    "esc_url_raw",
    "filter_input",
    "filter_var",
    "hash_equals",
    "is_email",
    "number_format",
    "sanitize_bookmark_field",
    "sanitize_bookmark",
    "sanitize_email",
    "sanitize_file_name",
    "sanitize_hex_color",
    "sanitize_hex_color_no_hash",
    "sanitize_html_class",
    "sanitize_meta",
    "sanitize_mime_type",
    "sanitize_option",
    "sanitize_sql_orderby",
    "sanitize_term_field",
    "sanitize_term",
    "sanitize_text_field",
    "sanitize_textarea_field",
    "sanitize_title",
    "sanitize_title_for_query",
    "sanitize_title_with_dashes",
    "sanitize_url",
    "sanitize_user",
    "sanitize_user_field",
    "validate_file",
    "wp_handle_sideload",
    "wp_handle_upload",
    "wp_kses",
    "wp_kses_allowed_html",
    "wp_kses_data",
    "wp_kses_one_attr",
    "wp_kses_post",
    "wp_parse_id_list",
    "wp_redirect",
    "wp_safe_redirect",
    "wp_sanitize_redirect",
    "wp_strip_all_tags",
];

/// Functions where only a specific argument position (0-indexed) is sanitized.
/// The third element is the parameter name for named-argument matching (PHP 8+).
/// For example, `sanitize_meta($meta_key, $meta_value, $meta_type)` only sanitizes
/// the second argument (`$meta_value`). All other functions in `SANITIZATION_FUNCTIONS`
/// are treated as sanitizing every argument.
const PARTIAL_SANITIZERS: &[(&str, usize, &str)] = &[
    ("sanitize_meta", 1, "meta_value"), // sanitize_meta($meta_key, $meta_value, $meta_type) — only $meta_value
    ("sanitize_option", 1, "value"),    // sanitize_option($option, $value) — only $value
    ("wp_kses", 0, "content"),          // wp_kses($content, $allowed_html, $allowed_protocols) — only $content
];

/// Sanitizers that implicitly handle slashes and do not require `wp_unslash()` wrapping.
const UNSLASHING_SANITIZERS: &[&str] = &[
    "absint",
    "boolval",
    "count",
    "doubleval",
    "floatval",
    "intval",
    "rest_sanitize_boolean",
    "sanitize_key",
    "sanitize_locale_name",
    "sizeof",
];

const TYPE_CHECK_FUNCTIONS: &[&str] = &[
    "is_array",
    "is_string",
    "is_int",
    "is_integer",
    "is_long",
    "is_float",
    "is_double",
    "is_real",
    "is_numeric",
    "is_bool",
    "is_null",
    "is_object",
    "is_resource",
    "is_callable",
];

const ARRAY_COMPARISON_FUNCTIONS: &[&str] = &["in_array", "array_search", "array_keys"];

const UNSLASHING_FUNCTIONS: &[&str] = &["wp_unslash", "stripslashes_deep", "stripslashes_from_strings_only"];

#[derive(Debug, Clone)]
pub struct ValidatedSanitizedInputRule {
    meta: &'static RuleMeta,
    cfg: ValidatedSanitizedInputConfig,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ValidatedSanitizedInputConfig {
    pub level: Level,
    /// Additional sanitization functions.
    pub custom_sanitization_functions: Vec<String>,
    /// Additional functions that both unslash and sanitize.
    pub custom_unslash_sanitization_functions: Vec<String>,
}

impl Default for ValidatedSanitizedInputConfig {
    fn default() -> Self {
        Self {
            level: Level::Error,
            custom_sanitization_functions: Vec::new(),
            custom_unslash_sanitization_functions: Vec::new(),
        }
    }
}

impl Config for ValidatedSanitizedInputConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ValidatedSanitizedInputRule {
    type Config = ValidatedSanitizedInputConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Validated Sanitized Input",
            code: "validated-sanitized-input",
            description: indoc! {"
                Detects use of superglobal variables (`$_GET`, `$_POST`, `$_REQUEST`,
                `$_COOKIE`, `$_SERVER`) that are not properly sanitized before use.

                All superglobal input must be sanitized with an appropriate sanitization
                function (e.g. `sanitize_text_field`) and unslashed with `wp_unslash()`
                before processing to prevent security vulnerabilities.
            "},
            good_example: indoc! {r"
                <?php

                function handle_input() {
                    if (isset($_POST['name'])) {
                        $name = sanitize_text_field(wp_unslash($_POST['name']));
                    }
                }
            "},
            bad_example: indoc! {r"
                <?php

                function handle_input() {
                    $name = $_POST['name'];
                }
            "},
            category: Category::Security,
            requirements: RuleRequirements::Integration(Integration::WordPress),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] =
            &[NodeKind::Function, NodeKind::Method, NodeKind::Closure, NodeKind::ArrowFunction];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config.clone() }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        // Build extended sanitization function list
        let custom_refs: Vec<&str> = self.cfg.custom_sanitization_functions.iter().map(|s| s.as_str()).collect();
        let custom_unslash_refs: Vec<&str> =
            self.cfg.custom_unslash_sanitization_functions.iter().map(|s| s.as_str()).collect();

        let all_sanitizers: Vec<&str> = SANITIZATION_FUNCTIONS
            .iter()
            .copied()
            .chain(UNSLASHING_SANITIZERS.iter().copied())
            .chain(custom_refs.iter().copied())
            .chain(custom_unslash_refs.iter().copied())
            .collect();

        let mut issues: Vec<Span> = Vec::new();

        match node {
            Node::ArrowFunction(arrow) => {
                collect_unsanitized_accesses(
                    ctx,
                    Node::Expression(arrow.expression),
                    &all_sanitizers,
                    &custom_unslash_refs,
                    &mut issues,
                    false,
                );
            }
            _ => {
                let statements: &[Statement<'arena>] = match node {
                    Node::Function(function) => function.body.statements.as_slice(),
                    Node::Method(method) => match &method.body {
                        MethodBody::Concrete(block) => block.statements.as_slice(),
                        MethodBody::Abstract(_) => return,
                    },
                    Node::Closure(closure) => closure.body.statements.as_slice(),
                    _ => return,
                };

                for stmt in statements {
                    collect_unsanitized_accesses(
                        ctx,
                        Node::Statement(stmt),
                        &all_sanitizers,
                        &custom_unslash_refs,
                        &mut issues,
                        false,
                    );
                }
            }
        }

        for span in issues {
            let issue = Issue::new(self.cfg.level(), "Superglobal input used without sanitization")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(span).with_message("Unsanitized superglobal access"),
                )
                .with_note("Superglobal input must be sanitized to prevent security vulnerabilities.")
                .with_help(
                    "Sanitize with e.g. `sanitize_text_field(wp_unslash(...))`, or use `absint()` / `intval()` for integers.",
                );

            ctx.collector.report(issue);
        }
    }
}

fn is_input_superglobal(name: &str) -> bool {
    INPUT_SUPERGLOBALS.iter().any(|sg| name.eq_ignore_ascii_case(sg))
}

fn is_superglobal_expr(expr: &Expression) -> bool {
    matches!(expr, Expression::Variable(Variable::Direct(var)) if is_input_superglobal(var.name))
}

fn is_wp_unslash_call<'arena>(ctx: &LintContext<'_, 'arena>, expr: &Expression<'arena>) -> bool {
    if let Expression::Call(Call::Function(fc)) = expr {
        function_call_matches_any(ctx, fc, UNSLASHING_FUNCTIONS).is_some()
    } else {
        false
    }
}

/// `sanitized` tracks whether we're inside a sanitization function call.
fn collect_unsanitized_accesses<'arena>(
    ctx: &LintContext<'_, 'arena>,
    node: Node<'_, 'arena>,
    sanitizers: &[&str],
    unslash_exempt: &[&str],
    issues: &mut Vec<Span>,
    sanitized: bool,
) {
    // Don't descend into nested function/method/closure definitions
    match node.kind() {
        NodeKind::Function | NodeKind::Method | NodeKind::Closure | NodeKind::ArrowFunction => return,
        _ => {}
    }

    // Exempt contexts: isset(), empty(), unset()
    match node {
        Node::IssetConstruct(_) | Node::EmptyConstruct(_) | Node::Unset(_) => {
            // These are validation/cleanup — exempt all superglobal accesses inside
            return;
        }
        _ => {}
    }

    // Check function calls that sanitize their arguments
    if let Node::FunctionCall(function_call) = node {
        // Unslashing functions are transparent — they only strip slashes, not sanitizers.
        // Pass through the current `sanitized` state to their arguments.
        if function_call_matches_any(ctx, function_call, UNSLASHING_FUNCTIONS).is_some() {
            for arg in &function_call.argument_list.arguments {
                collect_unsanitized_accesses(ctx, Node::Argument(arg), sanitizers, unslash_exempt, issues, sanitized);
            }
            return;
        }

        if let Some(matched_name) = function_call_matches_any(ctx, function_call, sanitizers) {
            // Numeric sanitizers and custom unslash+sanitize functions don't require wp_unslash() wrapping
            let needs_unslash = !UNSLASHING_SANITIZERS.iter().any(|n| matched_name.eq_ignore_ascii_case(n))
                && !unslash_exempt.iter().any(|n| matched_name.eq_ignore_ascii_case(n));

            // Check if this is a partial sanitizer (only specific argument positions are sanitized)
            let partial_info = PARTIAL_SANITIZERS
                .iter()
                .find(|(name, _, _)| matched_name.eq_ignore_ascii_case(name))
                .map(|(_, pos, param_name)| (*pos, *param_name));

            for (i, arg) in function_call.argument_list.arguments.iter().enumerate() {
                let arg_sanitized = match (partial_info, arg) {
                    // Named argument: match by parameter name, not position
                    (Some((_, param_name)), Argument::Named(named)) => {
                        named.name.value.eq_ignore_ascii_case(param_name)
                    }
                    // Positional argument: match by index
                    (Some((pos, _)), _) => i == pos,
                    // Not a partial sanitizer: all args are sanitized
                    (None, _) => true,
                };

                if arg_sanitized && needs_unslash {
                    // String sanitizer: only mark as sanitized if the argument is wrapped in wp_unslash()
                    let arg_value = match arg {
                        Argument::Positional(pos) => pos.value,
                        Argument::Named(named) => named.value,
                    };
                    let effectively_sanitized = is_wp_unslash_call(ctx, arg_value);
                    collect_unsanitized_accesses(
                        ctx,
                        Node::Argument(arg),
                        sanitizers,
                        unslash_exempt,
                        issues,
                        effectively_sanitized,
                    );
                } else {
                    collect_unsanitized_accesses(
                        ctx,
                        Node::Argument(arg),
                        sanitizers,
                        unslash_exempt,
                        issues,
                        arg_sanitized,
                    );
                }
            }
            return;
        }

        if function_call_matches_any(ctx, function_call, TYPE_CHECK_FUNCTIONS).is_some() {
            return;
        }

        if function_call_matches_any(ctx, function_call, ARRAY_COMPARISON_FUNCTIONS).is_some() {
            return;
        }
    }

    // Check for comparison operators — superglobals in comparisons are exempt
    if let Node::Binary(binary) = node
        && binary.operator.is_comparison()
    {
        return;
    }

    // Check for numeric casts (int), (float), (bool) — these sanitize
    if let Node::UnaryPrefix(unary) = node
        && matches!(
            unary.operator,
            UnaryPrefixOperator::IntCast(..)
                | UnaryPrefixOperator::IntegerCast(..)
                | UnaryPrefixOperator::FloatCast(..)
                | UnaryPrefixOperator::DoubleCast(..)
                | UnaryPrefixOperator::RealCast(..)
                | UnaryPrefixOperator::BoolCast(..)
                | UnaryPrefixOperator::BooleanCast(..)
        )
    {
        collect_unsanitized_accesses(ctx, Node::Expression(unary.operand), sanitizers, unslash_exempt, issues, true);
        return;
    }

    // Handle special binary operators
    if let Node::Binary(binary) = node
        && matches!(binary.operator, BinaryOperator::NullCoalesce(_))
    {
        collect_unsanitized_accesses(ctx, Node::Expression(binary.lhs), sanitizers, unslash_exempt, issues, sanitized);
        collect_unsanitized_accesses(ctx, Node::Expression(binary.rhs), sanitizers, unslash_exempt, issues, sanitized);
        return;
    }

    // Assignment to superglobal (LHS) — exempt
    if let Node::Assignment(assignment) = node
        && is_superglobal_expr(assignment.lhs)
    {
        collect_unsanitized_accesses(
            ctx,
            Node::Expression(assignment.rhs),
            sanitizers,
            unslash_exempt,
            issues,
            sanitized,
        );
        return;
    }

    // Superglobal array access — report if not sanitized
    if let Node::ArrayAccess(array_access) = node
        && !sanitized
        && is_superglobal_expr(array_access.array)
    {
        issues.push(array_access.span());
        return;
    }

    for child in node.children() {
        collect_unsanitized_accesses(ctx, child, sanitizers, unslash_exempt, issues, sanitized);
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::ValidatedSanitizedInputRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = validated_unslashed_sanitized,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_input() {
                if (isset($_POST['name'])) {
                    $name = sanitize_text_field(wp_unslash($_POST['name']));
                }
            }
        "}
    }

    test_lint_success! {
        name = absint_is_valid_sanitizer,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_input() {
                $id = absint($_POST['id']);
            }
        "}
    }

    test_lint_success! {
        name = intval_is_valid_sanitizer,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_input() {
                $id = intval($_GET['id']);
            }
        "}
    }

    test_lint_success! {
        name = int_cast_is_valid,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_input() {
                $id = (int) $_POST['id'];
            }
        "}
    }

    test_lint_failure! {
        name = no_sanitization,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_input() {
                $name = $_POST['name'];
            }
        "}
    }

    test_lint_success! {
        name = isset_is_not_flagged,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function check_input() {
                if (isset($_POST['name'])) {
                    // validation only
                }
            }
        "}
    }

    test_lint_success! {
        name = empty_is_not_flagged,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function check_input() {
                if (empty($_POST['name'])) {
                    return;
                }
            }
        "}
    }

    test_lint_success! {
        name = comparison_is_exempt,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function check_action() {
                if ($_POST['action'] === 'save') {
                    // comparison
                }
            }
        "}
    }

    test_lint_success! {
        name = type_check_is_exempt,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function check_items() {
                if (is_array($_POST['items'])) {
                    // type check
                }
            }
        "}
    }

    test_lint_failure! {
        name = server_var_needs_sanitization,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function get_host() {
                $host = $_SERVER['HTTP_HOST'];
            }
        "}
    }

    test_lint_failure! {
        name = cookie_needs_sanitization,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function get_session() {
                $session = $_COOKIE['session'];
            }
        "}
    }

    test_lint_success! {
        name = custom_sanitization_function,
        rule = ValidatedSanitizedInputRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.validated_sanitized_input.config.custom_sanitization_functions = vec!["my_sanitize".to_string()];
        },
        code = indoc! {r"
            <?php

            function handle_input() {
                $name = my_sanitize(wp_unslash($_POST['name']));
            }
        "}
    }

    test_lint_failure! {
        name = wp_unslash_alone_is_not_sanitizer,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_input() {
                $name = wp_unslash($_POST['name']);
            }
        "}
    }

    test_lint_failure! {
        name = wp_kses_allowed_html_arg_not_sanitized,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_content() {
                $content = wp_kses('safe content', $_POST['allowed_tags']);
            }
        "}
    }

    test_lint_success! {
        name = wp_kses_content_arg_is_sanitized,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_content() {
                $content = wp_kses(wp_unslash($_POST['content']), 'post');
            }
        "}
    }

    test_lint_success! {
        name = wp_kses_post_is_valid,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_content() {
                $content = wp_kses_post(wp_unslash($_POST['content']));
            }
        "}
    }

    test_lint_success! {
        name = sanitize_meta_value_arg_is_sanitized,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_meta() {
                $value = sanitize_meta('my_key', wp_unslash($_POST['value']), 'post');
            }
        "}
    }

    test_lint_failure! {
        name = sanitize_meta_key_arg_is_not_sanitized,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_meta() {
                $value = sanitize_meta($_POST['key'], 'safe_value', 'post');
            }
        "}
    }

    test_lint_success! {
        name = sanitize_option_value_arg_is_sanitized,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_option() {
                $value = sanitize_option('my_option', wp_unslash($_POST['value']));
            }
        "}
    }

    test_lint_failure! {
        name = sanitize_option_name_arg_is_not_sanitized,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_option() {
                $value = sanitize_option($_GET['option'], 'safe_value');
            }
        "}
    }

    test_lint_failure! {
        name = sanitize_without_unslash_is_flagged,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_input() {
                $name = sanitize_text_field($_POST['name']);
            }
        "}
    }

    test_lint_success! {
        name = custom_unslash_sanitizer_without_wp_unslash,
        rule = ValidatedSanitizedInputRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.validated_sanitized_input.config.custom_unslash_sanitization_functions = vec!["my_unslash_sanitize".to_string()];
        },
        code = indoc! {r"
            <?php

            function handle_input() {
                $name = my_unslash_sanitize($_POST['name']);
            }
        "}
    }

    test_lint_failure! {
        name = sanitize_option_reordered_named_args_option_not_sanitized,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_option() {
                $value = sanitize_option(value: 'safe', option: $_GET['option']);
            }
        "}
    }

    test_lint_success! {
        name = sanitize_option_reordered_named_args_value_is_sanitized,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_option() {
                $value = sanitize_option(option: 'my_option', value: wp_unslash($_POST['value']));
            }
        "}
    }

    test_lint_failure! {
        name = sanitize_meta_reordered_named_args_key_not_sanitized,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_meta() {
                $value = sanitize_meta(meta_value: 'safe', meta_key: $_POST['key'], meta_type: 'post');
            }
        "}
    }

    test_lint_success! {
        name = sanitize_key_no_unslash_needed,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_input() {
                $key = sanitize_key($_POST['key']);
            }
        "}
    }

    test_lint_success! {
        name = boolval_no_unslash_needed,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_input() {
                $flag = boolval($_POST['flag']);
            }
        "}
    }

    test_lint_success! {
        name = filter_input_is_valid_sanitizer,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_input() {
                $val = filter_input(INPUT_POST, 'field', FILTER_SANITIZE_STRING);
            }
        "}
    }

    test_lint_success! {
        name = wp_strip_all_tags_with_unslash,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_input() {
                $html = wp_strip_all_tags(wp_unslash($_POST['html']));
            }
        "}
    }

    test_lint_success! {
        name = stripslashes_deep_is_valid_unslash,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function handle_input() {
                $name = sanitize_text_field(stripslashes_deep($_POST['name']));
            }
        "}
    }

    test_lint_failure! {
        name = closure_without_sanitization,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            $handler = function() {
                $name = $_POST['name'];
            };
        "}
    }

    test_lint_success! {
        name = closure_with_sanitization,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            $handler = function() {
                $name = sanitize_text_field(wp_unslash($_POST['name']));
            };
        "}
    }

    test_lint_failure! {
        name = arrow_function_without_sanitization,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            $handler = fn() => $_POST['name'];
        "}
    }

    test_lint_success! {
        name = arrow_function_with_sanitization,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            $handler = fn() => sanitize_text_field(wp_unslash($_POST['name']));
        "}
    }
}
