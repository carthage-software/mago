use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::MethodCall;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches_any;
use crate::rule::utils::call::method_name_equals;
use crate::rule::utils::pest::find_expect_call;
use crate::rule::utils::pest::get_expect_argument;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

/// All function-to-matcher mappings for Pest expectations.
const FUNCTION_MATCHERS: &[(&str, &str)] = &[
    // Type checking functions
    ("is_array", "toBeArray"),
    ("is_string", "toBeString"),
    ("is_int", "toBeInt"),
    ("is_integer", "toBeInt"),
    ("is_float", "toBeFloat"),
    ("is_double", "toBeFloat"),
    ("is_bool", "toBeBool"),
    ("is_numeric", "toBeNumeric"),
    ("is_callable", "toBeCallable"),
    ("is_iterable", "toBeIterable"),
    ("is_object", "toBeObject"),
    ("is_resource", "toBeResource"),
    ("is_scalar", "toBeScalar"),
    ("is_null", "toBeNull"),
    // String functions
    ("str_starts_with", "toStartWith"),
    ("str_ends_with", "toEndWith"),
    ("ctype_alpha", "toBeAlpha"),
    ("ctype_alnum", "toBeAlphaNumeric"),
    // Array functions
    ("in_array", "toContain"),
    ("array_key_exists", "toHaveKey"),
    // File functions
    ("is_file", "toBeFile"),
    ("is_dir", "toBeDirectory"),
    ("is_readable", "toBeReadable"),
    ("is_writable", "toBeWritable"),
    ("file_exists", "toBeFile"),
    // Object functions
    ("property_exists", "toHaveProperty"),
];

#[derive(Debug, Clone)]
pub struct UseDedicatedExpectationRule {
    meta: &'static RuleMeta,
    cfg: UseDedicatedExpectationConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct UseDedicatedExpectationConfig {
    pub level: Level,
}

impl Default for UseDedicatedExpectationConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for UseDedicatedExpectationConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for UseDedicatedExpectationRule {
    type Config = UseDedicatedExpectationConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Use Dedicated Expectation",
            code: "use-dedicated-expectation",
            description: indoc! {"
                Use dedicated matchers instead of function calls in Pest tests.

                Instead of `expect(is_array($x))->toBeTrue()`, use `expect($x)->toBeArray()`.
                This provides clearer intent and better error messages.

                Supported patterns:
                - Type checks: is_array, is_string, is_int, is_float, is_bool, is_numeric, is_callable, is_iterable, is_object, is_resource, is_scalar, is_null
                - String: str_starts_with, str_ends_with, ctype_alpha, ctype_alnum
                - Array: in_array, array_key_exists
                - File: is_file, is_dir, is_readable, is_writable, file_exists
                - Object: property_exists
            "},
            good_example: indoc! {r#"
                <?php

                test('dedicated matchers', function () {
                    expect($value)->toBeArray();
                    expect($value)->toBeString();
                    expect($string)->toStartWith('prefix');
                    expect($array)->toContain($item);
                    expect($path)->toBeFile();
                    expect($obj)->toHaveProperty('name');
                });
            "#},
            bad_example: indoc! {r#"
                <?php

                test('function calls', function () {
                    expect(is_array($value))->toBeTrue();
                    expect(is_string($value))->toBeTrue();
                    expect(str_starts_with($string, 'prefix'))->toBeTrue();
                    expect(in_array($item, $array))->toBeTrue();
                    expect(is_file($path))->toBeTrue();
                    expect(property_exists($obj, 'name'))->toBeTrue();
                });
            "#},
            category: Category::Clarity,
            requirements: RuleRequirements::Integration(Integration::Pest),
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

        // Only interested in toBeTrue()
        if !method_name_equals(method_call, "toBeTrue") {
            return;
        }

        let Some(expect_call) = find_expect_call(method_call) else {
            return;
        };

        let Some(expect_arg) = get_expect_argument(ctx, expect_call) else {
            return;
        };

        // Check if the expect argument is a function call
        let Expression::Call(call) = expect_arg else {
            return;
        };

        let mago_syntax::ast::Call::Function(func_call) = call else {
            return;
        };

        // Get all function names to check
        let function_names: Vec<&str> = FUNCTION_MATCHERS.iter().map(|(name, _)| *name).collect();

        let Some(matched_name) = function_call_matches_any(ctx, func_call, &function_names) else {
            return;
        };

        // Find the corresponding matcher
        let Some((_, matcher)) = FUNCTION_MATCHERS.iter().find(|(name, _)| *name == matched_name) else {
            return;
        };

        self.report(ctx, method_call, expect_call, func_call, matched_name, matcher);
    }
}

impl UseDedicatedExpectationRule {
    fn report(
        &self,
        ctx: &mut LintContext<'_, '_>,
        method_call: &MethodCall<'_>,
        _expect_call: &FunctionCall<'_>,
        func_call: &FunctionCall<'_>,
        matched_name: &str,
        matcher: &str,
    ) {
        let issue =
            Issue::new(self.cfg.level(), format!("Use `{matcher}()` instead of `{matched_name}()` with `toBeTrue()`."))
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(method_call.span())
                        .with_message(format!("This can be simplified to `{matcher}()`.")),
                )
                .with_help(format!("Replace the function call with the dedicated Pest matcher `{matcher}()`."));

        // Get the function call arguments
        let args: Vec<_> = func_call.argument_list.arguments.iter().collect();

        match matched_name {
            // Single-arg functions: expect(func($x))->toBeTrue() -> expect($x)->matcher()
            // Strategy: Delete `funcname(`, delete function's `)`, replace `toBeTrue()` with matcher
            "is_array" | "is_string" | "is_int" | "is_integer" | "is_float" | "is_double" | "is_bool"
            | "is_numeric" | "is_callable" | "is_iterable" | "is_object" | "is_resource" | "is_scalar" | "is_null"
            | "is_file" | "is_dir" | "is_readable" | "is_writable" | "file_exists" | "ctype_alpha" | "ctype_alnum" => {
                if args.is_empty() {
                    ctx.collector.report(issue);
                    return;
                }

                // Span for `funcname(` - from function name start to after left paren
                let func_name_and_paren_span = func_call.function.span().join(func_call.argument_list.left_parenthesis);

                // Span for the function's closing paren
                let func_close_paren_span = func_call.argument_list.right_parenthesis;

                ctx.collector.propose(issue, |edits| {
                    // Delete `funcname(`
                    edits.push(TextEdit::delete(func_name_and_paren_span));
                    // Delete the function's closing `)`
                    edits.push(TextEdit::delete(func_close_paren_span));
                    // Replace `toBeTrue()` with the new matcher
                    edits.push(TextEdit::replace(
                        method_call.method.span().join(method_call.argument_list.span()),
                        format!("{matcher}()"),
                    ));
                });
            }
            // Two-arg same-order functions: expect(func($a, $b))->toBeTrue() -> expect($a)->matcher($b)
            // Strategy: Delete `funcname(`, replace `, ` with `)->matcher(`, delete `)->toBeTrue(`
            "str_starts_with" | "str_ends_with" | "property_exists" => {
                if args.len() < 2 {
                    ctx.collector.report(issue);
                    return;
                }

                // Span for `funcname(` - from function name start to after left paren
                let func_name_and_paren_span = func_call.function.span().join(func_call.argument_list.left_parenthesis);

                // Span from first arg end to second arg start (covers `, `)
                let first_arg_span = args[0].span();
                let second_arg_span = args[1].span();
                let comma_span = Span::new(first_arg_span.file_id, first_arg_span.end, second_arg_span.start);

                // Span for `)->toBeTrue(` (closing paren + arrow + method + open paren)
                let close_paren_to_method_open =
                    func_call.argument_list.right_parenthesis.join(method_call.argument_list.left_parenthesis);

                ctx.collector.propose(issue, |edits| {
                    // Delete `funcname(`
                    edits.push(TextEdit::delete(func_name_and_paren_span));
                    // Replace `, ` with `)->matcher(`
                    edits.push(TextEdit::replace(comma_span, format!(")->{matcher}(")));
                    // Delete `)->toBeTrue(`
                    edits.push(TextEdit::delete(close_paren_to_method_open));
                });
            }
            // Two-arg swapped functions: in_array($needle, $haystack), array_key_exists($key, $array)
            // These require argument swapping which we cannot do with precise edits
            // So we only report, no fix
            "in_array" | "array_key_exists" => {
                ctx.collector.report(issue);
            }

            _ => {
                ctx.collector.report(issue);
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    // Success cases
    test_lint_success! {
        name = dedicated_matchers,
        rule = UseDedicatedExpectationRule,
        code = indoc! {r#"
            <?php

            expect($value)->toBeArray();
            expect($value)->toBeString();
            expect($value)->toBeInt();
            expect($string)->toStartWith('prefix');
            expect($string)->toEndWith('suffix');
            expect($array)->toContain($item);
            expect($array)->toHaveKey('key');
            expect($path)->toBeFile();
            expect($path)->toBeDirectory();
            expect($obj)->toHaveProperty('name');
        "#}
    }

    test_lint_success! {
        name = non_matching_function_calls,
        rule = UseDedicatedExpectationRule,
        code = indoc! {r#"
            <?php

            expect(strlen($s) > 0)->toBeTrue();
            expect(custom_check($x))->toBeTrue();
            expect(count($array) > 0)->toBeTrue();
        "#}
    }

    // Type checking functions
    test_lint_failure! {
        name = is_array_to_be_true,
        rule = UseDedicatedExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(is_array($value))->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = is_string_to_be_true,
        rule = UseDedicatedExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(is_string($value))->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = is_int_to_be_true,
        rule = UseDedicatedExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(is_int($value))->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = is_integer_to_be_true,
        rule = UseDedicatedExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(is_integer($value))->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = is_null_to_be_true,
        rule = UseDedicatedExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(is_null($value))->toBeTrue();
        "#}
    }

    // String functions
    test_lint_failure! {
        name = str_starts_with_to_be_true,
        rule = UseDedicatedExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(str_starts_with($string, 'prefix'))->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = str_ends_with_to_be_true,
        rule = UseDedicatedExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(str_ends_with($string, 'suffix'))->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = ctype_alpha_to_be_true,
        rule = UseDedicatedExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(ctype_alpha($string))->toBeTrue();
        "#}
    }

    // Array functions
    test_lint_failure! {
        name = in_array_to_be_true,
        rule = UseDedicatedExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(in_array($item, $array))->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = array_key_exists_to_be_true,
        rule = UseDedicatedExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(array_key_exists('key', $array))->toBeTrue();
        "#}
    }

    // File functions
    test_lint_failure! {
        name = is_file_to_be_true,
        rule = UseDedicatedExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(is_file($path))->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = is_dir_to_be_true,
        rule = UseDedicatedExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(is_dir($path))->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = is_readable_to_be_true,
        rule = UseDedicatedExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(is_readable($path))->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = file_exists_to_be_true,
        rule = UseDedicatedExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(file_exists($path))->toBeTrue();
        "#}
    }

    // Object functions
    test_lint_failure! {
        name = property_exists_to_be_true,
        rule = UseDedicatedExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(property_exists($obj, 'name'))->toBeTrue();
        "#}
    }

    // Case insensitivity
    test_lint_failure! {
        name = case_insensitive_to_be_true,
        rule = UseDedicatedExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(is_array($value))->TOBETRUE();
        "#}
    }

    // Multiple violations
    test_lint_failure! {
        name = multiple_violations,
        rule = UseDedicatedExpectationRule,
        count = 5,
        code = indoc! {r#"
            <?php

            expect(is_array($a))->toBeTrue();
            expect(is_string($b))->toBeTrue();
            expect(str_starts_with($c, 'x'))->toBeTrue();
            expect(in_array($d, $arr))->toBeTrue();
            expect(is_file($e))->toBeTrue();
        "#}
    }
}
