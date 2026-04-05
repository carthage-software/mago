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
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::StringPart;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct BracedStringInterpolationRule {
    meta: &'static RuleMeta,
    cfg: BracedStringInterpolationConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct BracedStringInterpolationConfig {
    pub level: Level,
}

impl Default for BracedStringInterpolationConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for BracedStringInterpolationConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for BracedStringInterpolationRule {
    type Config = BracedStringInterpolationConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Braced String Interpolation",
            code: "braced-string-interpolation",
            description: indoc! {"
                Enforces the use of curly braces around variables within string interpolation.

                Using curly braces (`{$variable}`) within interpolated strings ensures clarity and avoids potential ambiguity,
                especially when variables are followed by alphanumeric characters. This rule promotes consistent and predictable code.
            "},
            good_example: indoc! {r#"
                <?php

                $a = "Hello, {$name}!";
                $b = "Hello, {$name}!";
                $c = "Hello, {$$name}!";
                $d = "Hello, {${$object->getMethod()}}!";
            "#},
            bad_example: indoc! {r#"
                <?php

                $a = "Hello, $name!";
                $b = "Hello, ${name}!";
                $c = "Hello, ${$name}!";
                $d = "Hello, ${$object->getMethod()}!";
            "#},
            category: Category::Consistency,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::CompositeString];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::CompositeString(composite_string) = node else {
            return;
        };

        let mut unbraced_expressions: Vec<(Span, Option<Span>)> = vec![];
        for part in composite_string.parts() {
            let StringPart::Expression(expression) = part else {
                continue;
            };

            let bareword_key_span = match expression {
                Expression::ArrayAccess(array_access) => {
                    if let Expression::Identifier(identifier) = array_access.index {
                        Some(identifier.span())
                    } else {
                        None
                    }
                }
                _ => None,
            };

            unbraced_expressions.push((expression.span(), bareword_key_span));
        }

        if unbraced_expressions.is_empty() {
            return;
        }

        let mut issue = Issue::new(self.cfg.level(), "Unbraced variable in string interpolation.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(composite_string.span())
                    .with_message("String interpolation contains unbraced variables"),
            );

        for (span, _) in &unbraced_expressions {
            issue = issue.with_annotation(
                Annotation::secondary(*span).with_message("Variable should be enclosed in curly braces"),
            );
        }

        issue = issue.with_note("Using curly braces around variables in interpolated strings improves readability and prevents potential parsing issues.")
            .with_help("Wrap the variable in curly braces, e.g., `{$variable}`.");

        ctx.collector.propose(issue, |edits| {
            for (span, bareword_key_span) in &unbraced_expressions {
                edits.push(TextEdit::insert(span.start_offset(), "{"));
                if let Some(key_span) = bareword_key_span {
                    edits.push(TextEdit::insert(key_span.start_offset(), "'"));
                    edits.push(TextEdit::insert(key_span.end_offset(), "'"));
                }
                edits.push(TextEdit::insert(span.end_offset(), "}"));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::BracedStringInterpolationRule;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_success! {
        name = already_braced_variable,
        rule = BracedStringInterpolationRule,
        code = indoc! {r#"
            <?php

            $name = 'world';
            echo "Hello, {$name}!";
        "#}
    }

    test_lint_success! {
        name = already_braced_array_access,
        rule = BracedStringInterpolationRule,
        code = indoc! {r#"
            <?php

            $o = ['user_id' => 77];
            echo "author-{$o['user_id']}";
        "#}
    }

    test_lint_success! {
        name = already_braced_property_access,
        rule = BracedStringInterpolationRule,
        code = indoc! {r#"
            <?php

            echo "value: {$obj->foo}";
        "#}
    }

    test_lint_success! {
        name = no_interpolation_literal_only,
        rule = BracedStringInterpolationRule,
        code = indoc! {r#"
            <?php

            echo "plain string with no interpolation";
        "#}
    }

    test_lint_failure! {
        name = unbraced_simple_variable,
        rule = BracedStringInterpolationRule,
        code = indoc! {r#"
            <?php

            $name = 'world';
            echo "Hello, $name!";
        "#}
    }

    test_lint_failure! {
        name = unbraced_array_access_bareword,
        rule = BracedStringInterpolationRule,
        code = indoc! {r#"
            <?php

            $o = ['user_id' => 77];
            echo "author-$o[user_id]";
        "#}
    }

    test_lint_failure! {
        name = unbraced_property_access,
        rule = BracedStringInterpolationRule,
        code = indoc! {r#"
            <?php

            echo "value: $obj->foo";
        "#}
    }

    test_lint_fix! {
        name = fix_bareword_array_key_is_quoted,
        rule = BracedStringInterpolationRule,
        code = indoc! {r#"
            <?php

            $o = ['user_id' => 77];
            print "hasauthorinfo author-$o[user_id]\n";
        "#},
        fixed = indoc! {r#"
            <?php

            $o = ['user_id' => 77];
            print "hasauthorinfo author-{$o['user_id']}\n";
        "#}
    }

    test_lint_fix! {
        name = fix_simple_variable,
        rule = BracedStringInterpolationRule,
        code = indoc! {r#"
            <?php

            $name = 'world';
            echo "Hello, $name!";
        "#},
        fixed = indoc! {r#"
            <?php

            $name = 'world';
            echo "Hello, {$name}!";
        "#}
    }

    test_lint_fix! {
        name = fix_array_access_with_numeric_index_is_not_quoted,
        rule = BracedStringInterpolationRule,
        code = indoc! {r#"
            <?php

            $o = [10, 20, 30];
            echo "first-$o[0]";
        "#},
        fixed = indoc! {r#"
            <?php

            $o = [10, 20, 30];
            echo "first-{$o[0]}";
        "#}
    }

    test_lint_fix! {
        name = fix_array_access_with_variable_index_is_not_quoted,
        rule = BracedStringInterpolationRule,
        code = indoc! {r#"
            <?php

            $o = [1 => 'a'];
            $i = 1;
            echo "val-$o[$i]";
        "#},
        fixed = indoc! {r#"
            <?php

            $o = [1 => 'a'];
            $i = 1;
            echo "val-{$o[$i]}";
        "#}
    }

    test_lint_fix! {
        name = fix_property_access,
        rule = BracedStringInterpolationRule,
        code = indoc! {r#"
            <?php

            echo "value: $obj->foo suffix";
        "#},
        fixed = indoc! {r#"
            <?php

            echo "value: {$obj->foo} suffix";
        "#}
    }

    test_lint_fix! {
        name = fix_multiple_interpolations_in_one_string,
        rule = BracedStringInterpolationRule,
        code = indoc! {r#"
            <?php

            $o = ['user_id' => 77];
            $name = 'alice';
            echo "user $name id $o[user_id] done";
        "#},
        fixed = indoc! {r#"
            <?php

            $o = ['user_id' => 77];
            $name = 'alice';
            echo "user {$name} id {$o['user_id']} done";
        "#}
    }

    test_lint_fix! {
        name = fix_bareword_key_in_heredoc,
        rule = BracedStringInterpolationRule,
        code = indoc! {r#"
            <?php

            $o = ['k' => 1];
            echo <<<EOT
            value $o[k] end
            EOT;
        "#},
        fixed = indoc! {r#"
            <?php

            $o = ['k' => 1];
            echo <<<EOT
            value {$o['k']} end
            EOT;
        "#}
    }

    test_lint_fix! {
        name = fix_back_to_back_interpolations,
        rule = BracedStringInterpolationRule,
        code = indoc! {r#"
            <?php

            $o = ['a' => 1, 'b' => 2];
            echo "$o[a]$o[b]";
        "#},
        fixed = indoc! {r#"
            <?php

            $o = ['a' => 1, 'b' => 2];
            echo "{$o['a']}{$o['b']}";
        "#}
    }
}
