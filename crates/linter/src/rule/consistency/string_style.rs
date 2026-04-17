use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct StringStyleRule {
    meta: &'static RuleMeta,
    cfg: StringStyleConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum StringStyleOption {
    Interpolation,
    Concatenation,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct StringStyleConfig {
    pub level: Level,
    pub style: StringStyleOption,
}

impl Default for StringStyleConfig {
    fn default() -> Self {
        Self { level: Level::Note, style: StringStyleOption::Interpolation }
    }
}

impl Config for StringStyleConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for StringStyleRule {
    type Config = StringStyleConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "String Style",
            code: "string-style",
            description: indoc! {"
                Enforces a consistent string style: either prefer string interpolation
                over concatenation, or prefer concatenation over interpolation.

                With `style: interpolation` (default), flags concatenation like
                `\"foo\" . $a . \"bar\"` and suggests `\"foo{$a}bar\"` instead.

                With `style: concatenation`, flags interpolation like `\"foo{$a}bar\"`
                and suggests `\"foo\" . $a . \"bar\"` instead.

                Only simple, interpolable expressions are considered: variables,
                property accesses, array accesses, and method calls. Concatenation
                involving function calls, static access, or complex expressions is
                never flagged.
            "},
            good_example: indoc! {r#"
                <?php

                // With the default `style: interpolation`:
                $a = "Hello, {$name}!";
                $b = "value: {$obj->name}";

                // Complex expressions stay as concatenation (never flagged):
                $c = "result: " . strtolower($input);
                $d = "class: " . Foo::class;
            "#},
            bad_example: indoc! {r#"
                <?php

                // With the default `style: interpolation`:
                $a = "Hello, " . $name . "!";
                $b = "value: " . $obj->name;
            "#},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Binary, NodeKind::CompositeString];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        match self.cfg.style {
            StringStyleOption::Interpolation => self.check_prefer_interpolation(ctx, node),
            StringStyleOption::Concatenation => self.check_prefer_concatenation(ctx, node),
        }
    }
}

impl StringStyleRule {
    fn check_prefer_interpolation<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Binary(binary) = node else {
            return;
        };

        if !binary.operator.is_concatenation() {
            return;
        }

        if let Some(Node::Binary(parent_binary)) = ctx.get_parent()
            && parent_binary.operator.is_concatenation()
        {
            return;
        }

        let parts = collect_concat_parts(binary.lhs, binary.rhs);

        let has_literal = parts.iter().any(|p| matches!(p, ConcatPart::Literal { .. }));
        let has_interpolable = parts.iter().any(|p| matches!(p, ConcatPart::Interpolable { .. }));
        if !has_literal || !has_interpolable {
            return;
        }

        let issue = Issue::new(self.cfg.level(), "String concatenation can be replaced with interpolation.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(binary.span())
                    .with_message("This concatenation contains expressions that support string interpolation"),
            )
            .with_help("Use a double-quoted string with `{$variable}` syntax instead of concatenation.");

        let all_fixable = parts
            .iter()
            .all(|p| matches!(p, ConcatPart::Literal { safe_to_double_quote: true, .. } | ConcatPart::Interpolable { .. }));

        if !all_fixable {
            ctx.collector.report(issue);
            return;
        }

        ctx.collector.propose(issue, |edits| {
            // Handle the start boundary: Make sure we have a double quoted string. If the first
            // part is an expression, it needs an opening `"{` so the interpolation becomes a valid
            // double-quoted string.
            match parts.first() {
                Some(ConcatPart::Interpolable { span }) => edits.push(TextEdit::insert(span.start_offset(), "\"{")),
                Some(ConcatPart::Literal { span, .. }) => edits.push(TextEdit::replace(span.start_offset()..span.start_offset()+1, "\"")),
                _ => {}
            };

            // Handle the end boundary: Make sure we have a double quoted string. If the last part
            // is an expression, it needs a closing `}"`.
            match parts.last() {
                Some(ConcatPart::Interpolable { span }) => edits.push(TextEdit::insert(span.end_offset(), "}\"")),
                Some(ConcatPart::Literal { span, .. }) => edits.push(TextEdit::replace(span.end_offset()-1..span.end_offset(), "\"")),
                _ => {}
            };

            // Handle each boundary between two consecutive parts. The "gap" between them in
            // source code covers the closing quote (if the left side is a literal), the ` . `
            // concatenation operator with any surrounding whitespace, and the opening quote
            // (if the right side is a literal). We replace the entire gap with the appropriate
            // interpolation glue.
            for pair in parts.windows(2) {
                let (curr, next) = (&pair[0], &pair[1]);

                let (gap_start, gap_end, glue) = match (curr, next) {
                    (ConcatPart::Literal { span: lhs, .. }, ConcatPart::Literal { span: rhs, .. }) => {
                        // Two literals concatenated: fuse them by removing the closing quote of
                        // the left, the concat operator, and the opening quote of the right.
                        (lhs.end_offset() - 1, rhs.start_offset() + 1, "")
                    }
                    (ConcatPart::Literal { span: lhs, .. }, ConcatPart::Interpolable { span: rhs }) => {
                        // Literal -> expression: strip left literal's closing quote and the
                        // concat operator, then open interpolation with `{`.
                        (lhs.end_offset() - 1, rhs.start_offset(), "{")
                    }
                    (ConcatPart::Interpolable { span: lhs }, ConcatPart::Literal { span: rhs, .. }) => {
                        // Expression -> literal: close interpolation with `}` and strip the
                        // concat operator and right literal's opening quote.
                        (lhs.end_offset(), rhs.start_offset() + 1, "}")
                    }
                    (ConcatPart::Interpolable { span: lhs }, ConcatPart::Interpolable { span: rhs }) => {
                        // Expression -> expression: close previous interpolation and open the
                        // next one back-to-back.
                        (lhs.end_offset(), rhs.start_offset(), "}{")
                    }
                    _ => unreachable!("`all_fixable` excludes `ConcatPart::Other` branches"),
                };

                edits.push(TextEdit::replace(gap_start..gap_end, glue));
            }
        });
    }

    fn check_prefer_concatenation<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::CompositeString(CompositeString::Interpolated(interpolated_string)) = node else {
            return;
        };

        let issue = Issue::new(self.cfg.level(), "String interpolation can be replaced with concatenation.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(interpolated_string.span())
                    .with_message("This string uses interpolation instead of concatenation"),
            )
            .with_help("Use the `.` concatenation operator with single-quoted strings instead of interpolation.");

        let parts: Vec<&StringPart<'arena>> = interpolated_string
            .parts
            .iter()
            .filter(|part| !matches!(part, StringPart::Literal(literal) if literal.value.is_empty()))
            .collect();
        if parts.is_empty() {
            ctx.collector.report(issue);
            return;
        }

        let left_quote = interpolated_string.left_double_quote;
        let right_quote = interpolated_string.right_double_quote;
        let first_is_literal = matches!(parts.first(), Some(StringPart::Literal(_)));
        let last_is_literal = matches!(parts.last(), Some(StringPart::Literal(_)));

        ctx.collector.propose(issue, |edits| {
            if !first_is_literal {
                let drop_end = match parts.first() {
                    Some(StringPart::BracedExpression(braced)) => braced.left_brace.end_offset(),
                    _ => left_quote.end_offset(),
                };

                edits.push(TextEdit::delete(left_quote.start_offset()..drop_end));
            }

            if !last_is_literal {
                let drop_start = match parts.last() {
                    Some(StringPart::BracedExpression(braced)) => braced.right_brace.start_offset(),
                    _ => right_quote.start_offset(),
                };

                edits.push(TextEdit::delete(drop_start..right_quote.end_offset()));
            }

            for pair in parts.windows(2) {
                let (curr, next) = (&pair[0], &pair[1]);
                edits.push(boundary_edit_for_to_concat(curr, next));
            }
        });
    }
}

enum ConcatPart {
    Literal { span: Span, safe_to_double_quote: bool },
    Interpolable { span: Span },
    Other,
}

fn collect_concat_parts<'arena>(lhs: &Expression<'arena>, rhs: &Expression<'arena>) -> Vec<ConcatPart> {
    let mut parts = Vec::new();
    collect_concat_side(lhs, &mut parts);
    collect_concat_side(rhs, &mut parts);
    parts
}

fn collect_concat_side<'arena>(expr: &Expression<'arena>, parts: &mut Vec<ConcatPart>) {
    match expr {
        Expression::Binary(binary) if binary.operator.is_concatenation() => {
            collect_concat_side(binary.lhs, parts);
            collect_concat_side(binary.rhs, parts);
        }
        Expression::Literal(Literal::String(literal)) => {
            let safe_to_double_quote = matches!(literal.kind, Some(LiteralStringKind::DoubleQuoted)) || !literal.raw.contains(['\\', '$', '"']);
            parts.push(ConcatPart::Literal { span: literal.span, safe_to_double_quote });
        }
        expr if is_interpolable(expr) => {
            parts.push(ConcatPart::Interpolable { span: expr.span() });
        }
        _ => {
            parts.push(ConcatPart::Other);
        }
    }
}

fn is_interpolable(expr: &Expression) -> bool {
    match expr {
        Expression::Variable(_) => true,
        Expression::Access(Access::Property(access)) => is_interpolable(access.object),
        Expression::Access(Access::NullSafeProperty(access)) => is_interpolable(access.object),
        Expression::ArrayAccess(access) => is_interpolable(access.array),
        Expression::Call(Call::Method(call)) => is_interpolable(call.object),
        Expression::Call(Call::NullSafeMethod(call)) => is_interpolable(call.object),
        _ => false,
    }
}

/// Computes the edit that replaces the "gap" between two consecutive `StringPart`s of an
/// interpolated string with the appropriate concatenation glue.
///
/// The gap range depends on whether each side is a literal (no braces), a plain expression,
/// or a braced expression. When a side is a braced expression, the `{` or `}` delimiter
/// is absorbed into the replaced gap so the resulting code has no stray braces.
fn boundary_edit_for_to_concat<'arena>(curr: &StringPart<'arena>, next: &StringPart<'arena>) -> TextEdit {
    let gap_start = match curr {
        StringPart::Literal(literal) => literal.span.end_offset(),
        StringPart::Expression(expr) => expr.end_offset(),
        StringPart::BracedExpression(braced) => braced.right_brace.start_offset(),
    };

    let gap_end = match next {
        StringPart::Literal(literal) => literal.span.start_offset(),
        StringPart::Expression(expr) => expr.start_offset(),
        StringPart::BracedExpression(braced) => braced.left_brace.end_offset(),
    };

    let glue = match (curr, next) {
        (StringPart::Literal(_), StringPart::Expression(_) | StringPart::BracedExpression(_)) => "\" . ",
        (StringPart::Expression(_) | StringPart::BracedExpression(_), StringPart::Literal(_)) => " . \"",
        (
            StringPart::Expression(_) | StringPart::BracedExpression(_),
            StringPart::Expression(_) | StringPart::BracedExpression(_),
        ) => " . ",
        (StringPart::Literal(_), StringPart::Literal(_)) => "\" . \"",
    };

    TextEdit::replace(gap_start..gap_end, glue)
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    use super::*;

    fn prefer_interpolation(s: &mut Settings) {
        s.rules.string_style.config.style = StringStyleOption::Interpolation;
    }

    fn prefer_concatenation(s: &mut Settings) {
        s.rules.string_style.config.style = StringStyleOption::Concatenation;
    }

    test_lint_success! {
        name = interpolation_already_used,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            $name = "world";
            echo "Hello, {$name}!";
        "#}
    }

    test_lint_success! {
        name = concat_with_non_interpolable_function_call,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "result: " . strtolower("FOO");
        "#}
    }

    test_lint_success! {
        name = concat_with_static_access,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "class: " . Foo::class;
        "#}
    }

    test_lint_success! {
        name = concat_of_two_literals_only,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "foo" . "bar";
        "#}
    }

    test_lint_success! {
        name = concat_of_two_variables_no_literal,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo $a . $b;
        "#}
    }

    test_lint_success! {
        name = concat_with_arithmetic,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "total: " . ($a + $b);
        "#}
    }

    test_lint_failure! {
        name = simple_variable_concat,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "Hello, " . $name . "!";
        "#}
    }

    test_lint_failure! {
        name = property_access_concat,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "value: " . $obj->name;
        "#}
    }

    test_lint_failure! {
        name = variable_prefix_concat,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo $name . " is here";
        "#}
    }

    test_lint_failure! {
        name = method_call_concat,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "result: " . $obj->getName();
        "#}
    }

    test_lint_success! {
        name = concat_with_enum_case_property_not_interpolable,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "Hello " . SomeEnum::World->value;
        "#}
    }

    test_lint_success! {
        name = concat_with_class_constant_property_not_interpolable,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "greeting: " . Foo::BAR->name;
        "#}
    }

    test_lint_success! {
        name = concat_with_static_method_call_not_interpolable,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "result: " . Foo::bar();
        "#}
    }

    test_lint_success! {
        name = concat_with_function_call_not_interpolable,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "result: " . strtolower($x);
        "#}
    }

    test_lint_success! {
        name = concat_with_static_property_access_not_interpolable,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "value: " . Foo::$bar;
        "#}
    }

    test_lint_success! {
        name = concat_already_used,
        rule = StringStyleRule,
        settings = prefer_concatenation,
        code = indoc! {r#"
            <?php

            echo "Hello, " . $name . "!";
        "#}
    }

    test_lint_success! {
        name = plain_double_quoted_string,
        rule = StringStyleRule,
        settings = prefer_concatenation,
        code = indoc! {r#"
            <?php

            echo "no interpolation here";
        "#}
    }

    test_lint_failure! {
        name = simple_interpolation,
        rule = StringStyleRule,
        settings = prefer_concatenation,
        code = indoc! {r#"
            <?php

            echo "Hello, {$name}!";
        "#}
    }

    test_lint_failure! {
        name = property_interpolation,
        rule = StringStyleRule,
        settings = prefer_concatenation,
        code = indoc! {r#"
            <?php

            echo "value: {$obj->name}";
        "#}
    }

    test_lint_fix! {
        name = fix_simple_variable_suffix,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "Hello, " . $name . "!";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "Hello, {$name}!";
        "#}
    }

    test_lint_fix! {
        name = fix_leading_expression,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo $name . " is here";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "{$name} is here";
        "#}
    }

    test_lint_fix! {
        name = fix_trailing_expression,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "hello " . $name;
        "#},
        fixed = indoc! {r#"
            <?php

            echo "hello {$name}";
        "#}
    }

    test_lint_fix! {
        name = fix_property_access,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "value: " . $obj->name;
        "#},
        fixed = indoc! {r#"
            <?php

            echo "value: {$obj->name}";
        "#}
    }

    test_lint_fix! {
        name = fix_method_call,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "result: " . $obj->getName() . ".";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "result: {$obj->getName()}.";
        "#}
    }

    test_lint_fix! {
        name = fix_nested_property_chain,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "deep: " . $a->b->c->d;
        "#},
        fixed = indoc! {r#"
            <?php

            echo "deep: {$a->b->c->d}";
        "#}
    }

    test_lint_fix! {
        name = fix_null_safe_property_chain,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "maybe: " . $obj?->inner?->name;
        "#},
        fixed = indoc! {r#"
            <?php

            echo "maybe: {$obj?->inner?->name}";
        "#}
    }

    test_lint_fix! {
        name = fix_array_access,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "user " . $users[0] . " logged in";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "user {$users[0]} logged in";
        "#}
    }

    test_lint_fix! {
        name = fix_multiple_expressions,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "hi " . $first . " " . $last . "!";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "hi {$first} {$last}!";
        "#}
    }

    test_lint_fix! {
        name = fix_adjacent_expressions,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "key=" . $a . $b . " done";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "key={$a}{$b} done";
        "#}
    }

    test_lint_fix! {
        name = fix_newlines_between_parts,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "Hello, "
                . $name
                . "!";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "Hello, {$name}!";
        "#}
    }

    test_lint_fix! {
        name = fix_preserves_double_quote_escapes,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "he said \"hi\" to " . $name;
        "#},
        fixed = indoc! {r#"
            <?php

            echo "he said \"hi\" to {$name}";
        "#}
    }

    test_lint_fix! {
        name = fix_expression_then_expression_then_literal,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo $first . $last . " done";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "{$first}{$last} done";
        "#}
    }

    test_lint_fix! {
        name = fix_literal_then_expression_then_expression,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo "start: " . $first . $last;
        "#},
        fixed = indoc! {r#"
            <?php

            echo "start: {$first}{$last}";
        "#}
    }

    test_lint_success! {
        name = single_quoted_literal_not_fixed,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo 'foo' . 'bar';
        "#}
    }

    test_lint_fix! {
        name = fix_single_quoted_no_unsafe_chars,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo 'Hello, ' . $name . '!';
        "#},
        fixed = indoc! {r#"
            <?php

            echo "Hello, {$name}!";
        "#}
    }

    test_lint_fix! {
        name = fix_single_quoted_only_prefix,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo 'prefix: ' . $value;
        "#},
        fixed = indoc! {r#"
            <?php

            echo "prefix: {$value}";
        "#}
    }

    test_lint_fix! {
        name = fix_single_quoted_only_suffix,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo $name . ' is here';
        "#},
        fixed = indoc! {r#"
            <?php

            echo "{$name} is here";
        "#}
    }

    test_lint_failure! {
        name = single_quoted_with_backslash_not_fixable,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo 'Hello\n' . $name;
        "#}
    }

    test_lint_failure! {
        name = single_quoted_with_dollar_sign_not_fixable,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo '$foo ' . $name;
        "#}
    }

    test_lint_failure! {
        name = single_quoted_with_double_quote_not_fixable,
        rule = StringStyleRule,
        settings = prefer_interpolation,
        code = indoc! {r#"
            <?php

            echo 'say "hi" to ' . $name;
        "#}
    }

    test_lint_fix! {
        name = fix_interp_simple_variable,
        rule = StringStyleRule,
        settings = prefer_concatenation,
        code = indoc! {r#"
            <?php

            echo "Hello, {$name}!";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "Hello, " . $name . "!";
        "#}
    }

    test_lint_fix! {
        name = fix_interp_unbraced_variable,
        rule = StringStyleRule,
        settings = prefer_concatenation,
        code = indoc! {r#"
            <?php

            echo "Hello, $name!";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "Hello, " . $name . "!";
        "#}
    }

    test_lint_fix! {
        name = fix_interp_property_access,
        rule = StringStyleRule,
        settings = prefer_concatenation,
        code = indoc! {r#"
            <?php

            echo "value: {$obj->name}";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "value: " . $obj->name;
        "#}
    }

    test_lint_fix! {
        name = fix_interp_leading_expression,
        rule = StringStyleRule,
        settings = prefer_concatenation,
        code = indoc! {r#"
            <?php

            echo "{$name} is here";
        "#},
        fixed = indoc! {r#"
            <?php

            echo $name . " is here";
        "#}
    }

    test_lint_fix! {
        name = fix_interp_trailing_expression,
        rule = StringStyleRule,
        settings = prefer_concatenation,
        code = indoc! {r#"
            <?php

            echo "hello {$name}";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "hello " . $name;
        "#}
    }

    test_lint_fix! {
        name = fix_interp_multiple_expressions,
        rule = StringStyleRule,
        settings = prefer_concatenation,
        code = indoc! {r#"
            <?php

            echo "hi {$first} {$last}!";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "hi " . $first . " " . $last . "!";
        "#}
    }

    test_lint_fix! {
        name = fix_interp_adjacent_expressions,
        rule = StringStyleRule,
        settings = prefer_concatenation,
        code = indoc! {r#"
            <?php

            echo "key={$a}{$b} done";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "key=" . $a . $b . " done";
        "#}
    }

    test_lint_fix! {
        name = fix_interp_only_expression,
        rule = StringStyleRule,
        settings = prefer_concatenation,
        code = indoc! {r#"
            <?php

            echo "{$name}";
        "#},
        fixed = indoc! {r#"
            <?php

            echo $name;
        "#}
    }

    test_lint_fix! {
        name = fix_interp_preserves_escapes,
        rule = StringStyleRule,
        settings = prefer_concatenation,
        code = indoc! {r#"
            <?php

            echo "he said \"hi\" to {$name}";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "he said \"hi\" to " . $name;
        "#}
    }

    test_lint_fix! {
        name = fix_interp_array_access,
        rule = StringStyleRule,
        settings = prefer_concatenation,
        code = indoc! {r#"
            <?php

            echo "user {$users[0]} logged in";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "user " . $users[0] . " logged in";
        "#}
    }
}
