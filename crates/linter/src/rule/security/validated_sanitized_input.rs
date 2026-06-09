use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
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

const PARTIAL_SANITIZERS: &[(&str, usize, &str)] =
    &[("sanitize_meta", 1, "meta_value"), ("sanitize_option", 1, "value"), ("wp_kses", 0, "content")];

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

const VERIFICATION_FUNCTIONS: &[&str] = &["wp_verify_nonce", "check_admin_referer", "check_ajax_referer"];

const UNSLASHING_FUNCTIONS: &[&str] = &["wp_unslash", "stripslashes_deep", "stripslashes_from_strings_only"];

#[derive(Debug, Clone)]
pub struct ValidatedSanitizedInputRule {
    meta: &'static RuleMeta,
    cfg: ValidatedSanitizedInputConfig,
}

#[derive(Debug, Clone, Eq, PartialEq, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct ValidatedSanitizedInputConfig {
    pub level: Level,
    pub custom_sanitization_functions: Vec<String>,
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

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        match node {
            Node::ArrowFunction(arrow) => {
                self.collect_unsanitized_accesses(ctx, Node::Expression(arrow.expression), false);
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
                    self.collect_unsanitized_accesses(ctx, Node::Statement(stmt), false);
                }
            }
        }
    }
}

fn is_input_superglobal(name: &[u8]) -> bool {
    INPUT_SUPERGLOBALS.iter().any(|sg| name.eq_ignore_ascii_case(sg.as_bytes()))
}

fn is_superglobal_expr(expr: &Expression) -> bool {
    matches!(expr, Expression::Variable(Variable::Direct(var)) if is_input_superglobal(var.name))
}

fn is_wp_unslash_call<'arena, A>(ctx: &LintContext<'_, 'arena, A>, expr: &Expression<'arena>) -> bool
where
    A: Arena,
{
    if let Expression::Call(Call::Function(fc)) = expr {
        function_call_matches_any(ctx, fc, UNSLASHING_FUNCTIONS).is_some()
    } else {
        false
    }
}

impl ValidatedSanitizedInputRule {
    fn collect_unsanitized_accesses<'arena, A>(
        &self,
        ctx: &mut LintContext<'_, 'arena, A>,
        node: Node<'_, 'arena>,
        sanitized: bool,
    ) where
        A: Arena,
    {
        match node.kind() {
            NodeKind::Function | NodeKind::Method | NodeKind::Closure | NodeKind::ArrowFunction => return,
            _ => {}
        }

        match node {
            Node::IssetConstruct(_) | Node::EmptyConstruct(_) | Node::Unset(_) => return,
            _ => {}
        }

        if let Node::FunctionCall(function_call) = node {
            if function_call_matches_any(ctx, function_call, UNSLASHING_FUNCTIONS).is_some() {
                for arg in &function_call.argument_list.arguments {
                    self.collect_unsanitized_accesses(ctx, Node::Argument(arg), sanitized);
                }
                return;
            }

            let custom_san = self.cfg.custom_sanitization_functions.as_slice();
            let custom_unslash = self.cfg.custom_unslash_sanitization_functions.as_slice();
            let matched = function_call_matches_any(ctx, function_call, SANITIZATION_FUNCTIONS)
                .map(|n| (n, false))
                .or_else(|| function_call_matches_any(ctx, function_call, UNSLASHING_SANITIZERS).map(|n| (n, true)))
                .or_else(|| function_call_matches_any(ctx, function_call, custom_san).map(|n| (n, false)))
                .or_else(|| function_call_matches_any(ctx, function_call, custom_unslash).map(|n| (n, true)));

            if let Some((matched_name, is_unslash_exempt)) = matched {
                let needs_unslash = !is_unslash_exempt;

                let partial_info = PARTIAL_SANITIZERS
                    .iter()
                    .find(|(name, _, _)| matched_name.eq_ignore_ascii_case(name))
                    .map(|(_, pos, param_name)| (*pos, *param_name));

                for (i, arg) in function_call.argument_list.arguments.iter().enumerate() {
                    let arg_sanitized = match (partial_info, arg) {
                        (Some((_, param_name)), Argument::Named(named)) => {
                            named.name.value.eq_ignore_ascii_case(param_name.as_bytes())
                        }
                        (Some((pos, _)), _) => i == pos,
                        (None, _) => true,
                    };

                    if arg_sanitized && needs_unslash {
                        let arg_value = match arg {
                            Argument::Positional(pos) => pos.value,
                            Argument::Named(named) => named.value,
                        };
                        let effectively_sanitized = is_wp_unslash_call(ctx, arg_value);
                        self.collect_unsanitized_accesses(ctx, Node::Argument(arg), effectively_sanitized);
                    } else {
                        self.collect_unsanitized_accesses(ctx, Node::Argument(arg), arg_sanitized);
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

            if function_call_matches_any(ctx, function_call, VERIFICATION_FUNCTIONS).is_some() {
                return;
            }
        }

        if let Node::Binary(binary) = node
            && binary.operator.is_comparison()
        {
            return;
        }

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
            self.collect_unsanitized_accesses(ctx, Node::Expression(unary.operand), true);
            return;
        }

        if let Node::Binary(binary) = node
            && matches!(binary.operator, BinaryOperator::NullCoalesce(_))
        {
            self.collect_unsanitized_accesses(ctx, Node::Expression(binary.lhs), sanitized);
            self.collect_unsanitized_accesses(ctx, Node::Expression(binary.rhs), sanitized);
            return;
        }

        if let Node::Assignment(assignment) = node
            && is_superglobal_expr(assignment.lhs)
        {
            self.collect_unsanitized_accesses(ctx, Node::Expression(assignment.rhs), sanitized);
            return;
        }

        if let Node::ArrayAccess(array_access) = node
            && !sanitized
            && is_superglobal_expr(array_access.array)
        {
            let issue = Issue::new(self.cfg.level(), "Superglobal input used without sanitization")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(array_access.span()).with_message("Unsanitized superglobal access"),
                )
                .with_note("Superglobal input must be sanitized to prevent security vulnerabilities.")
                .with_help(
                    "Sanitize with e.g. `sanitize_text_field(wp_unslash(...))`, or use `absint()` / `intval()` for integers.",
                );
            ctx.collector.report(issue);
            return;
        }

        for child in node.children() {
            self.collect_unsanitized_accesses(ctx, child, sanitized);
        }
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

    test_lint_success! {
        name = wp_verify_nonce_does_not_flag,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function process_form() {
                if (!wp_verify_nonce($_POST['_wpnonce'], 'my_action')) {
                    wp_die('Security check failed');
                }
            }
        "}
    }

    test_lint_success! {
        name = check_admin_referer_does_not_flag,
        rule = ValidatedSanitizedInputRule,
        code = indoc! {r"
            <?php

            function process_admin() {
                check_admin_referer('my_action', $_POST['_wpnonce']);
            }
        "}
    }
}
