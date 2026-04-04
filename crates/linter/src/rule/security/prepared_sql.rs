use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Access;
use mago_syntax::ast::Argument;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::Call;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Literal;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::StringPart;
use mago_syntax::ast::UnaryPrefixOperator;
use mago_syntax::ast::Variable;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches_any;
use crate::rule::utils::call::method_name_matches_any;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const QUERY_METHODS: &[&str] = &["query", "get_var", "get_col", "get_row", "get_results", "prepare"];

const SAFE_ESCAPING_FUNCTIONS: &[&str] = &["absint", "intval", "floatval"];

const FORMATTING_FUNCTIONS: &[&str] = &["sprintf", "vsprintf", "wp_sprintf", "implode", "join"];

#[derive(Debug, Clone)]
pub struct PreparedSqlRule {
    meta: &'static RuleMeta,
    cfg: PreparedSqlConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PreparedSqlConfig {
    pub level: Level,
}

impl Default for PreparedSqlConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for PreparedSqlConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreparedSqlRule {
    type Config = PreparedSqlConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prepared SQL",
            code: "prepared-sql",
            description: indoc! {"
                Detects `$wpdb` query method calls where the SQL string contains variables
                not passed through `$wpdb->prepare()`.

                All dynamic values in SQL queries must use `$wpdb->prepare()` with placeholders
                to prevent SQL injection vulnerabilities. Only literal strings and `$wpdb` table
                properties (e.g. `$wpdb->posts`) are safe without preparation.
            "},
            good_example: indoc! {r#"
                <?php

                $wpdb->query($wpdb->prepare("DELETE FROM {$wpdb->posts} WHERE ID = %d", $post_id));
            "#},
            bad_example: indoc! {r#"
                <?php

                $wpdb->query("DELETE FROM {$wpdb->posts} WHERE ID = $post_id");
            "#},
            category: Category::Security,
            requirements: RuleRequirements::Integration(Integration::WordPress),
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

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::MethodCall(method_call) = node else {
            return;
        };

        // Check if this is a method call on $wpdb
        if !is_wpdb_variable(method_call.object) {
            return;
        }

        // Check if this is a query method
        let Some(matched_method) = method_name_matches_any(method_call, QUERY_METHODS) else {
            return;
        };

        // Get the SQL argument: prefer a named `query:` argument (PHP 8 named args can
        // appear in any order), then fall back to the first positional/named argument.
        let Some(sql_expr) = method_call
            .argument_list
            .arguments
            .iter()
            .find_map(|arg| match arg {
                Argument::Named(named) if named.name.value.eq_ignore_ascii_case("query") => Some(named.value),
                _ => None,
            })
            .or_else(|| {
                method_call.argument_list.arguments.first().map(|arg| match arg {
                    Argument::Positional(arg) => arg.value,
                    Argument::Named(arg) => arg.value,
                })
            })
        else {
            return;
        };

        // Check if the argument is already wrapped in $wpdb->prepare()
        // (skip this check when we're already inspecting the prepare() call itself)
        if !matched_method.eq_ignore_ascii_case("prepare") && is_wpdb_prepare_call(sql_expr) {
            return;
        }

        // Check if the SQL expression contains unsafe variables
        if !contains_unsafe_variables(ctx, sql_expr) {
            return;
        }

        let issue = Issue::new(self.cfg.level(), "SQL query contains variables not passed through `$wpdb->prepare()`")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(method_call.span()).with_message("Unprepared SQL query with dynamic values"),
            )
            .with_note(
                "Dynamic values in SQL queries must use `$wpdb->prepare()` with placeholders to prevent SQL injection.",
            )
            .with_help("Use `$wpdb->prepare()` with `%s`, `%d`, or `%f` placeholders for dynamic values.");

        ctx.collector.report(issue);
    }
}

/// Check if an expression is a reference to the $wpdb variable.
fn is_wpdb_variable(expr: &Expression) -> bool {
    matches!(expr, Expression::Variable(Variable::Direct(var)) if var.name == "$wpdb")
}

/// Check if an expression is a call to `$wpdb->prepare(...)`.
fn is_wpdb_prepare_call(expr: &Expression) -> bool {
    if let Expression::Call(Call::Method(method_call)) = expr
        && is_wpdb_variable(method_call.object)
        && let ClassLikeMemberSelector::Identifier(ident) = &method_call.method
    {
        return ident.value.eq_ignore_ascii_case("prepare");
    }

    false
}

/// Check if an expression contains unsafe (non-$wpdb, non-escaped) variables.
fn contains_unsafe_variables(ctx: &LintContext, expr: &Expression) -> bool {
    match expr {
        // Single-quoted string literals have no interpolation — always safe
        Expression::Literal(Literal::String(_)) => false,
        // Numeric literals are safe
        Expression::Literal(Literal::Integer(_) | Literal::Float(_)) => false,
        // Interpolated strings — check each part for non-$wpdb variables
        Expression::CompositeString(composite_string) => {
            for part in composite_string.parts() {
                match part {
                    StringPart::Literal(_) => {}
                    StringPart::Expression(expr) => {
                        if contains_unsafe_variables(ctx, expr) {
                            return true;
                        }
                    }
                    StringPart::BracedExpression(braced) => {
                        if contains_unsafe_variables(ctx, braced.expression) {
                            return true;
                        }
                    }
                }
            }

            false
        }
        // String concatenation — check both sides
        Expression::Binary(binary) if matches!(binary.operator, BinaryOperator::StringConcat(_)) => {
            contains_unsafe_variables(ctx, binary.lhs) || contains_unsafe_variables(ctx, binary.rhs)
        }
        // A call to $wpdb->prepare() is safe
        _ if is_wpdb_prepare_call(expr) => false,
        // Formatting functions: check their arguments for unsafe variables
        Expression::Call(Call::Function(function_call))
            if function_call_matches_any(ctx, function_call, FORMATTING_FUNCTIONS).is_some() =>
        {
            for arg in function_call.argument_list.arguments.iter() {
                let arg_expr = match arg {
                    Argument::Positional(pos) => pos.value,
                    Argument::Named(named) => named.value,
                };
                if contains_unsafe_variables(ctx, arg_expr) {
                    return true;
                }
            }
            false
        }
        // A call to a safe escaping function is safe
        Expression::Call(Call::Function(function_call)) => {
            function_call_matches_any(ctx, function_call, SAFE_ESCAPING_FUNCTIONS).is_none()
        }
        // A numeric cast is safe: (int), (float)
        Expression::UnaryPrefix(unary)
            if matches!(
                unary.operator,
                UnaryPrefixOperator::IntCast(..)
                    | UnaryPrefixOperator::IntegerCast(..)
                    | UnaryPrefixOperator::FloatCast(..)
                    | UnaryPrefixOperator::DoubleCast(..)
                    | UnaryPrefixOperator::RealCast(..)
                    | UnaryPrefixOperator::BoolCast(..)
                    | UnaryPrefixOperator::BooleanCast(..)
            ) =>
        {
            false
        }
        // Parenthesized expressions
        Expression::Parenthesized(paren) => contains_unsafe_variables(ctx, paren.expression),
        // A static `$wpdb->property` or `$wpdb?->property` access (e.g., `$wpdb->posts`) is safe,
        // but dynamic access like `$wpdb->$table_name` is not, as the property name could be user-controlled.
        Expression::Access(Access::Property(property_access))
            if is_wpdb_variable(property_access.object)
                && matches!(property_access.property, ClassLikeMemberSelector::Identifier(_)) =>
        {
            false
        }
        Expression::Access(Access::NullSafeProperty(property_access))
            if is_wpdb_variable(property_access.object)
                && matches!(property_access.property, ClassLikeMemberSelector::Identifier(_)) =>
        {
            false
        }
        // Any other expression (variable, method call, etc.) is unsafe
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::PreparedSqlRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = prepared_query,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->query($wpdb->prepare("DELETE FROM {$wpdb->posts} WHERE ID = %d", $post_id));
        "#}
    }

    test_lint_success! {
        name = literal_string_query,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->query("SELECT * FROM my_table WHERE 1=1");
        "#}
    }

    test_lint_success! {
        name = wpdb_table_property_in_string,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->get_results("SELECT * FROM {$wpdb->posts}");
        "#}
    }

    test_lint_success! {
        name = absint_escaped_variable,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->query("SELECT * FROM {$wpdb->posts} WHERE ID = " . absint($id));
        "#}
    }

    test_lint_success! {
        name = intval_escaped_variable,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->query("SELECT * FROM {$wpdb->posts} WHERE ID = " . intval($id));
        "#}
    }

    test_lint_failure! {
        name = esc_sql_not_sufficient,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->query("SELECT * FROM {$wpdb->posts} WHERE post_name = '" . esc_sql($name) . "'");
        "#}
    }

    test_lint_success! {
        name = int_cast_is_safe,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->query("SELECT * FROM {$wpdb->posts} WHERE ID = " . (int) $id);
        "#}
    }

    test_lint_failure! {
        name = variable_interpolation_in_query,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->query("SELECT * FROM {$wpdb->posts} WHERE ID = $id");
        "#}
    }

    test_lint_failure! {
        name = variable_concatenation_in_query,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->query("SELECT * FROM {$wpdb->posts} WHERE ID = " . $id);
        "#}
    }

    test_lint_failure! {
        name = raw_variable_as_query,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->query($sql);
        "#}
    }

    test_lint_failure! {
        name = get_var_needs_prepare,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->get_var("SELECT COUNT(*) FROM {$wpdb->posts} WHERE post_author = $author_id");
        "#}
    }

    test_lint_failure! {
        name = get_results_needs_prepare,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->get_results("SELECT * FROM {$wpdb->posts} WHERE post_author = $author_id");
        "#}
    }

    test_lint_failure! {
        name = get_row_needs_prepare,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->get_row("SELECT * FROM {$wpdb->posts} WHERE ID = $id");
        "#}
    }

    test_lint_failure! {
        name = get_col_needs_prepare,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->get_col("SELECT ID FROM {$wpdb->posts} WHERE post_author = $author_id");
        "#}
    }

    test_lint_failure! {
        name = dynamic_wpdb_property_is_unsafe,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->query("SELECT * FROM " . $wpdb->$table_name);
        "#}
    }

    test_lint_failure! {
        name = named_argument_needs_prepare,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->query(query: "SELECT * FROM {$wpdb->posts} WHERE ID = $id");
        "#}
    }

    test_lint_failure! {
        name = reordered_named_args_query_still_detected,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->get_results(output: OBJECT, query: "SELECT * FROM {$wpdb->posts} WHERE ID = $id");
        "#}
    }

    test_lint_success! {
        name = reordered_named_args_prepared_query_is_safe,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->get_results(output: OBJECT, query: $wpdb->prepare("SELECT * FROM {$wpdb->posts} WHERE ID = %d", $id));
        "#}
    }

    test_lint_failure! {
        name = variable_in_prepare_template,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->prepare("SELECT * FROM $table WHERE id = %d", $id);
        "#}
    }

    test_lint_success! {
        name = sprintf_with_safe_args,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->query(sprintf("SELECT * FROM {$wpdb->posts} WHERE id = %d", absint($id)));
        "#}
    }

    test_lint_failure! {
        name = sprintf_with_unsafe_args,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->query(sprintf("SELECT * FROM %s WHERE id = %d", $table, $id));
        "#}
    }

    test_lint_failure! {
        name = like_escape_is_not_safe,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->query("SELECT * FROM {$wpdb->posts} WHERE post_name LIKE '%" . like_escape($search) . "%'");
        "#}
    }

    test_lint_success! {
        name = nullsafe_wpdb_property_is_safe,
        rule = PreparedSqlRule,
        code = indoc! {r#"
            <?php

            $wpdb->get_results("SELECT * FROM {$wpdb?->posts}");
        "#}
    }
}
