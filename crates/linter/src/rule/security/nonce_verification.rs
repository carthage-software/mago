use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
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

const DEFAULT_NONCE_FUNCTIONS: &[&str] = &["wp_verify_nonce", "check_admin_referer", "check_ajax_referer"];

const NONCE_SUPERGLOBALS: &[&str] = &["$_POST", "$_GET", "$_REQUEST", "$_FILES"];

const EXEMPT_FUNCTIONS: &[&str] = &[
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
    "in_array",
    "array_search",
    "array_keys",
    "wp_verify_nonce",
    "check_admin_referer",
    "check_ajax_referer",
];

#[derive(Debug, Clone)]
pub struct NonceVerificationRule {
    meta: &'static RuleMeta,
    cfg: NonceVerificationConfig,
}

#[derive(Debug, Clone, Eq, PartialEq, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NonceVerificationConfig {
    pub level: Level,
    pub custom_nonce_functions: Vec<String>,
}

impl Default for NonceVerificationConfig {
    fn default() -> Self {
        Self { level: Level::Warning, custom_nonce_functions: Vec::new() }
    }
}

impl Config for NonceVerificationConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NonceVerificationRule {
    type Config = NonceVerificationConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Nonce Verification",
            code: "nonce-verification",
            description: indoc! {"
                Detects access to `$_POST`, `$_GET`, `$_REQUEST`, or `$_FILES` superglobals
                inside a function or method that does not call a WordPress nonce verification
                function (`wp_verify_nonce`, `check_admin_referer`, or `check_ajax_referer`).

                Nonce verification is essential to protect against Cross-Site Request Forgery
                (CSRF) attacks. All form and request data processing should verify the nonce first.
            "},
            good_example: indoc! {r"
                <?php

                function process_form() {
                    if (!wp_verify_nonce($_POST['_wpnonce'], 'my_action')) {
                        wp_die('Security check failed');
                    }
                    $name = sanitize_text_field(wp_unslash($_POST['name']));
                    update_option('name', $name);
                }
            "},
            bad_example: indoc! {r"
                <?php

                function process_form() {
                    $name = $_POST['name'];
                    update_option('name', $name);
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
        let custom_nonce = self.cfg.custom_nonce_functions.as_slice();

        if let Node::ArrowFunction(arrow) = node {
            self.collect_and_report(ctx, Node::Expression(arrow.expression), custom_nonce, false);
            return;
        }

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
            self.collect_and_report(ctx, Node::Statement(stmt), custom_nonce, false);

            if node_tree_contains_nonce_call(ctx, Node::Statement(stmt), custom_nonce) {
                return;
            }
        }
    }
}

impl NonceVerificationRule {
    fn collect_and_report<'arena, A>(
        &self,
        ctx: &mut LintContext<'_, 'arena, A>,
        node: Node<'_, 'arena>,
        custom_nonce_functions: &[String],
        exempt: bool,
    ) where
        A: Arena,
    {
        match node.kind() {
            NodeKind::Function | NodeKind::Method | NodeKind::Closure | NodeKind::ArrowFunction => return,
            _ => {}
        }

        match node {
            Node::IssetConstruct(isset) => {
                for value in &isset.values {
                    self.collect_and_report(ctx, Node::Expression(value), custom_nonce_functions, true);
                }
                return;
            }
            Node::EmptyConstruct(empty) => {
                self.collect_and_report(ctx, Node::Expression(empty.value), custom_nonce_functions, true);
                return;
            }
            Node::Unset(unset) => {
                for value in &unset.values {
                    self.collect_and_report(ctx, Node::Expression(value), custom_nonce_functions, true);
                }
                return;
            }
            _ => {}
        }

        if let Node::FunctionCall(function_call) = node
            && (function_call_matches_any(ctx, function_call, EXEMPT_FUNCTIONS).is_some()
                || function_call_matches_any(ctx, function_call, custom_nonce_functions).is_some())
        {
            return;
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
            return;
        }

        if let Node::ArrayAccess(array_access) = node
            && !exempt
            && is_superglobal_expr(array_access.array)
        {
            let issue = Issue::new(self.cfg.level(), "Processing form data without nonce verification")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(array_access.span())
                        .with_message("Superglobal accessed without nonce verification"),
                )
                .with_note("Nonce verification is essential to protect against CSRF attacks.")
                .with_help(
                    "Call `wp_verify_nonce()`, `check_admin_referer()`, or `check_ajax_referer()` before processing form data.",
                );
            ctx.collector.report(issue);
            return;
        }

        if let Node::Assignment(assignment) = node
            && is_superglobal_expr(assignment.lhs)
        {
            self.collect_and_report(ctx, Node::Expression(assignment.rhs), custom_nonce_functions, false);
            return;
        }

        for child in node.children() {
            self.collect_and_report(ctx, child, custom_nonce_functions, exempt);
        }
    }
}

fn is_nonce_superglobal(name: &[u8]) -> bool {
    NONCE_SUPERGLOBALS.iter().any(|sg| name.eq_ignore_ascii_case(sg.as_bytes()))
}

fn is_superglobal_expr(expr: &Expression) -> bool {
    matches!(expr, Expression::Variable(Variable::Direct(var)) if is_nonce_superglobal(var.name))
}

fn node_tree_contains_nonce_call<'arena, A>(
    ctx: &LintContext<'_, 'arena, A>,
    node: Node<'_, 'arena>,
    custom_nonce_functions: &[String],
) -> bool
where
    A: Arena,
{
    if let Node::FunctionCall(function_call) = node
        && (function_call_matches_any(ctx, function_call, DEFAULT_NONCE_FUNCTIONS).is_some()
            || function_call_matches_any(ctx, function_call, custom_nonce_functions).is_some())
    {
        return true;
    }

    match node.kind() {
        NodeKind::Function | NodeKind::Method | NodeKind::Closure | NodeKind::ArrowFunction => {
            return false;
        }
        _ => {}
    }

    for child in node.children() {
        if node_tree_contains_nonce_call(ctx, child, custom_nonce_functions) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NonceVerificationRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = nonce_before_post_access,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            function process_form() {
                wp_verify_nonce($_POST['_wpnonce'], 'my_action');
                $name = sanitize_text_field($_POST['name']);
            }
        "}
    }

    test_lint_success! {
        name = check_admin_referer_is_valid,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            function process_form() {
                check_admin_referer('my_action');
                $name = $_POST['name'];
            }
        "}
    }

    test_lint_success! {
        name = check_ajax_referer_is_valid,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            function handle_ajax() {
                check_ajax_referer('my_action', 'nonce');
                $data = $_POST['data'];
            }
        "}
    }

    test_lint_failure! {
        name = post_access_without_nonce,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            function process_form() {
                $name = $_POST['name'];
                update_option('name', $name);
            }
        "}
    }

    test_lint_failure! {
        name = get_access_without_nonce,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            function process_form() {
                $page = $_GET['page'];
            }
        "}
    }

    test_lint_failure! {
        name = request_access_without_nonce,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            function process_form() {
                $action = $_REQUEST['action'];
            }
        "}
    }

    test_lint_failure! {
        name = files_access_without_nonce,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            function upload_handler() {
                $file = $_FILES['upload'];
            }
        "}
    }

    test_lint_failure! {
        name = post_access_before_nonce,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            function process_form() {
                $name = $_POST['name'];
                wp_verify_nonce($_POST['_wpnonce'], 'my_action');
            }
        "}
    }

    test_lint_success! {
        name = isset_check_is_exempt,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            function check_form() {
                if (isset($_POST['name'])) {
                    // just checking existence
                }
            }
        "}
    }

    test_lint_success! {
        name = empty_check_is_exempt,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            function check_form() {
                if (empty($_POST['name'])) {
                    return;
                }
            }
        "}
    }

    test_lint_success! {
        name = unset_is_exempt,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            function cleanup() {
                unset($_POST['temp']);
            }
        "}
    }

    test_lint_success! {
        name = assignment_overwrite_is_exempt,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            function reset_post() {
                $_POST = [];
            }
        "}
    }

    test_lint_success! {
        name = custom_nonce_function,
        rule = NonceVerificationRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.nonce_verification.config.custom_nonce_functions = vec!["my_verify_nonce".to_string()];
        },
        code = indoc! {r"
            <?php

            function process_form() {
                my_verify_nonce('my_action');
                $name = $_POST['name'];
            }
        "}
    }

    test_lint_success! {
        name = type_test_is_exempt,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            function check_type() {
                if (is_array($_POST['items'])) {
                    // type check only
                }
            }
        "}
    }

    test_lint_success! {
        name = comparison_is_exempt,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            function check_action() {
                if ($_POST['action'] === 'save') {
                    // comparison only
                }
            }
        "}
    }

    test_lint_success! {
        name = in_array_is_exempt,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            function check_valid() {
                if (in_array($_POST['action'], ['save', 'delete'], true)) {
                    // array comparison only
                }
            }
        "}
    }

    test_lint_failure! {
        name = sanitize_without_nonce_is_flagged,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            function sanitize_only() {
                $name = sanitize_text_field($_POST['name']);
            }
        "}
    }

    test_lint_success! {
        name = closure_with_nonce,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            $handler = function() {
                wp_verify_nonce($_POST['_wpnonce'], 'action');
                $data = $_POST['data'];
            };
        "}
    }

    test_lint_failure! {
        name = closure_without_nonce,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            $handler = function() {
                $data = $_POST['data'];
            };
        "}
    }

    test_lint_failure! {
        name = arrow_fn_without_nonce,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            $handler = fn() => process_post($_POST['x']);
        "}
    }

    test_lint_success! {
        name = arrow_fn_with_isset,
        rule = NonceVerificationRule,
        code = indoc! {r"
            <?php

            $check = fn() => isset($_POST['x']);
        "}
    }
}
