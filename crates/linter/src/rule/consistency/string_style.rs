use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

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

        let has_literal = parts.iter().any(|p| matches!(p, ConcatPart::StringLiteral));
        let has_interpolable = parts.iter().any(|p| matches!(p, ConcatPart::Interpolable));
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

        ctx.collector.report(issue);
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

        ctx.collector.report(issue);
    }
}

enum ConcatPart {
    StringLiteral,
    Interpolable,
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
        Expression::Literal(Literal::String(_)) => {
            parts.push(ConcatPart::StringLiteral);
        }
        expr if is_interpolable(expr) => {
            parts.push(ConcatPart::Interpolable);
        }
        _ => {
            parts.push(ConcatPart::Other);
        }
    }
}

fn is_interpolable(expr: &Expression) -> bool {
    match expr {
        Expression::Variable(_) => true,
        Expression::Access(access) => {
            matches!(access, Access::Property(_) | Access::NullSafeProperty(_))
        }
        Expression::ArrayAccess(_) => true,
        Expression::Call(call) => {
            matches!(call, Call::Method(_))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::settings::Settings;
    use crate::test_lint_failure;
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
}
