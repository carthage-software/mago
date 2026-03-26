use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_docblock::document::TagKind;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::TriviaKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoShorthandArrayTypeRule {
    meta: &'static RuleMeta,
    cfg: NoShorthandArrayTypeConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoShorthandArrayTypeConfig {
    pub level: Level,
}

impl Default for NoShorthandArrayTypeConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoShorthandArrayTypeConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

/// Tag kinds that contain type information in their description.
fn is_typed_tag(kind: TagKind) -> bool {
    matches!(
        kind,
        TagKind::Param
            | TagKind::Return
            | TagKind::Var
            | TagKind::Property
            | TagKind::PropertyRead
            | TagKind::PropertyWrite
            | TagKind::PhpstanParam
            | TagKind::PhpstanReturn
            | TagKind::PhpstanVar
            | TagKind::PsalmParam
            | TagKind::PsalmReturn
            | TagKind::PsalmVar
            | TagKind::PsalmProperty
            | TagKind::PsalmPropertyRead
            | TagKind::PsalmPropertyWrite
            | TagKind::ParamOut
            | TagKind::PsalmParamOut
    )
}

/// Finds all `[]` occurrences in a string that are preceded by a type name character.
fn find_shorthand_array_offsets(text: &str) -> Vec<usize> {
    let bytes = text.as_bytes();
    let mut offsets = Vec::new();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'[' && bytes[i + 1] == b']' {
            // Check if preceded by a type name character (alphanumeric, underscore, backslash,
            // or closing bracket for nested arrays like `string[][]`).
            if i > 0 {
                let prev = bytes[i - 1];
                if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'\\' || prev == b']' {
                    offsets.push(i);
                }
            }
            i += 2;
        } else {
            i += 1;
        }
    }
    offsets
}

/// Array-like type names that should always have an explicit key type.
const ARRAY_LIKE_TYPES: &[&str] = &["array", "non-empty-array"];

/// Finds single-argument generic array types like `array<Foo>` or `non-empty-array<Foo>`.
///
/// These should use explicit key types: `array<array-key, Foo>` or `array<int, Foo>`.
/// Returns a list of `(start_offset, end_offset)` pairs for the matched type name + `<...>`.
fn find_single_arg_array_generics(text: &str) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    let lower = text.to_ascii_lowercase();

    for type_name in ARRAY_LIKE_TYPES {
        let mut search_from = 0;
        while let Some(pos) = lower[search_from..].find(type_name) {
            let abs_pos = search_from + pos;
            let after_name = abs_pos + type_name.len();

            // Must be preceded by a non-alphanumeric char (or start of string) to avoid
            // matching inside other identifiers.
            if abs_pos > 0 {
                let prev = text.as_bytes()[abs_pos - 1];
                if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'-' {
                    search_from = after_name;
                    continue;
                }
            }

            // Must be followed by `<`.
            if after_name >= text.len() || text.as_bytes()[after_name] != b'<' {
                search_from = after_name;
                continue;
            }

            // Find the matching `>`, counting nested `<>` pairs.
            let mut depth = 1u32;
            let mut i = after_name + 1;
            let bytes = text.as_bytes();
            while i < bytes.len() && depth > 0 {
                match bytes[i] {
                    b'<' => depth += 1,
                    b'>' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }

            if depth == 0 {
                // Check that the content between `<` and `>` has no comma (single argument).
                let inner = &text[after_name + 1..i - 1];
                if !inner.contains(',') {
                    results.push((abs_pos, i));
                }
            }

            search_from = if depth == 0 { i } else { after_name };
        }
    }
    results
}

impl LintRule for NoShorthandArrayTypeRule {
    type Config = NoShorthandArrayTypeConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Shorthand Array Type",
            code: "no-shorthand-array-type",
            description: indoc! {"
                Disallows imprecise array type syntax in docblock type annotations.

                This rule flags two patterns:
                - `T[]` shorthand syntax (e.g., `string[]`)
                - Single-argument generic array syntax (e.g., `array<string>`, `non-empty-array<Foo>`)

                Both forms omit the key type, making them semantically vague. They do not
                communicate whether the developer expects a sequential list or a specific key-value
                map. Using explicit `list<T>` or `array<K, V>` types leads to better-documented
                and safer codebases.
            "},
            good_example: indoc! {r#"
                <?php

                /**
                 * @param list<User> $users
                 * @return array<int, string>
                 */
                function process(array $users): array {
                    return [];
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                /**
                 * @param User[] $users
                 * @return array<string>
                 */
                function process(array $users): array {
                    return [];
                }
            "#},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Program];
        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        for trivia in &program.trivia {
            if trivia.kind != TriviaKind::DocBlockComment {
                continue;
            }

            let Ok(document) = mago_docblock::parse_trivia(ctx.arena, trivia) else {
                continue;
            };

            for tag in document.get_tags() {
                if !is_typed_tag(tag.kind) {
                    continue;
                }

                let description = tag.description;

                // Check for T[] shorthand syntax.
                for offset in find_shorthand_array_offsets(description) {
                    let bracket_span = tag.description_span.subspan(offset as u32, (offset + 2) as u32);

                    ctx.collector.report(
                        Issue::new(self.cfg.level, "Shorthand array type `T[]` detected in docblock.")
                            .with_code(self.meta.code)
                            .with_annotation(
                                Annotation::primary(bracket_span).with_message("Shorthand `[]` array syntax"),
                            )
                            .with_annotation(
                                Annotation::secondary(tag.span).with_message("In this tag"),
                            )
                            .with_note("The `T[]` syntax is semantically vague — it does not distinguish between `list<T>` and `array<array-key, T>`.")
                            .with_help("Use `list<T>` for sequential arrays or `array<K, V>` for key-value maps."),
                    );
                }

                // Check for single-argument generic array syntax like `array<Foo>`.
                for (start, end) in find_single_arg_array_generics(description) {
                    let type_span = tag.description_span.subspan(start as u32, end as u32);
                    let matched = &description[start..end];

                    ctx.collector.report(
                        Issue::new(self.cfg.level, format!("Single-argument array type `{matched}` detected in docblock."))
                            .with_code(self.meta.code)
                            .with_annotation(
                                Annotation::primary(type_span).with_message("Missing explicit key type"),
                            )
                            .with_annotation(
                                Annotation::secondary(tag.span).with_message("In this tag"),
                            )
                            .with_note("Single-argument array generics omit the key type, making them semantically vague.")
                            .with_help("Use `list<T>` for sequential arrays or `array<K, V>` / `non-empty-array<K, V>` for key-value maps."),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoShorthandArrayTypeRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = explicit_array_syntax,
        rule = NoShorthandArrayTypeRule,
        code = indoc! {r#"
            <?php

            /**
             * @param array<int, User> $users
             */
            function foo(array $users): void {
            }
        "#}
    }

    test_lint_success! {
        name = explicit_list_syntax,
        rule = NoShorthandArrayTypeRule,
        code = indoc! {r#"
            <?php

            /**
             * @param list<User> $users
             */
            function foo(array $users): void {
            }
        "#}
    }

    test_lint_success! {
        name = no_docblock,
        rule = NoShorthandArrayTypeRule,
        code = indoc! {r#"
            <?php

            function foo(array $users): void {
            }
        "#}
    }

    test_lint_failure! {
        name = shorthand_param_type,
        rule = NoShorthandArrayTypeRule,
        code = indoc! {r#"
            <?php

            /**
             * @param string[] $items
             */
            function foo(array $items): void {
            }
        "#}
    }

    test_lint_failure! {
        name = shorthand_return_type,
        rule = NoShorthandArrayTypeRule,
        code = indoc! {r#"
            <?php

            /**
             * @return User[]
             */
            function foo(): array {
                return [];
            }
        "#}
    }

    test_lint_failure! {
        name = shorthand_var_type,
        rule = NoShorthandArrayTypeRule,
        code = indoc! {r#"
            <?php

            /** @var int[] */
            $items = [1, 2, 3];
        "#}
    }

    test_lint_failure! {
        name = multiple_shorthand_in_one_docblock,
        rule = NoShorthandArrayTypeRule,
        count = 2,
        code = indoc! {r#"
            <?php

            /**
             * @param string[] $a
             * @param int[] $b
             */
            function foo(array $a, array $b): void {
            }
        "#}
    }

    test_lint_failure! {
        name = nested_shorthand_array,
        rule = NoShorthandArrayTypeRule,
        count = 2,
        code = indoc! {r#"
            <?php

            /**
             * @param string[][] $matrix
             */
            function foo(array $matrix): void {
            }
        "#}
    }

    test_lint_success! {
        name = explicit_two_arg_non_empty_array,
        rule = NoShorthandArrayTypeRule,
        code = indoc! {r#"
            <?php

            /**
             * @param non-empty-array<int, string> $items
             */
            function foo(array $items): void {
            }
        "#}
    }

    test_lint_failure! {
        name = single_arg_array_generic,
        rule = NoShorthandArrayTypeRule,
        code = indoc! {r#"
            <?php

            /**
             * @param array<string> $items
             */
            function foo(array $items): void {
            }
        "#}
    }

    test_lint_failure! {
        name = single_arg_non_empty_array_generic,
        rule = NoShorthandArrayTypeRule,
        code = indoc! {r#"
            <?php

            /**
             * @param non-empty-array<Foo> $items
             */
            function foo(array $items): void {
            }
        "#}
    }

    test_lint_failure! {
        name = single_arg_array_in_return,
        rule = NoShorthandArrayTypeRule,
        code = indoc! {r#"
            <?php

            /**
             * @return array<string>
             */
            function foo(): array {
                return [];
            }
        "#}
    }

    test_lint_failure! {
        name = mixed_shorthand_and_single_arg,
        rule = NoShorthandArrayTypeRule,
        count = 2,
        code = indoc! {r#"
            <?php

            /**
             * @param string[] $a
             * @return array<Foo>
             */
            function foo(array $a): array {
                return [];
            }
        "#}
    }
}
