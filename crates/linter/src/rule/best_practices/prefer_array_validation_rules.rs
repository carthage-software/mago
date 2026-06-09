use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Literal;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::scope::FunctionLikeScope;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferArrayValidationRulesRule {
    meta: &'static RuleMeta,
    cfg: PreferArrayValidationRulesConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct PreferArrayValidationRulesConfig {
    pub level: Level,
}

impl Default for PreferArrayValidationRulesConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for PreferArrayValidationRulesConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferArrayValidationRulesRule {
    type Config = PreferArrayValidationRulesConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Array Validation Rules",
            code: "prefer-array-validation-rules",
            description: indoc! {"
                Detects `|`-delimited validation rule strings in a `rules()` method. Expressing the rules
                as an array keeps a consistent structure and avoids ambiguity when a rule value itself
                contains a `|`, such as a `regex` pattern.
            "},
            good_example: indoc! {r"
                <?php

                class StoreUserRequest
                {
                    public function rules(): array
                    {
                        return [
                            'name' => ['required', 'string', 'max:255'],
                        ];
                    }
                }
            "},
            bad_example: indoc! {r"
                <?php

                class StoreUserRequest
                {
                    public function rules(): array
                    {
                        return [
                            'name' => 'required|string|max:255',
                        ];
                    }
                }
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::Integration(Integration::Laravel),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::KeyValueArrayElement];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::KeyValueArrayElement(element) = node else {
            return;
        };

        // Only inside a `rules()` method, the Laravel form-request convention.
        let Some(FunctionLikeScope::Method(method_name, _)) = ctx.scope.get_function_like_scope() else {
            return;
        };

        if !method_name.eq_ignore_ascii_case(b"rules") {
            return;
        }

        let Expression::Literal(Literal::String(string)) = element.value else {
            return;
        };

        let Some(content) = string.value else {
            return;
        };

        // Only multi-rule strings; a single rule needs no splitting.
        if !content.contains(&b'|') {
            return;
        }

        // `regex` and `not_regex` patterns may contain a literal `|`, so the string cannot be split safely.
        if memchr::memmem::find(content, b"regex:").is_some() {
            return;
        }

        let segments: Vec<&[u8]> = content.split(|&byte| byte == b'|').collect();

        // Bail on empty segments (leading, trailing, or doubled `|`) and on segments that would
        // need escaping inside a single-quoted string.
        if segments.iter().any(|segment| segment.is_empty() || segment.contains(&b'\'') || segment.contains(&b'\\')) {
            return;
        }

        let mut replacement = String::from("[");
        for (index, segment) in segments.iter().enumerate() {
            if index > 0 {
                replacement.push_str(", ");
            }

            replacement.push('\'');
            replacement.push_str(&String::from_utf8_lossy(segment));
            replacement.push('\'');
        }

        replacement.push(']');

        let issue = Issue::new(
            self.cfg.level(),
            "Validation rules should be declared as an array, not a `|`-delimited string.",
        )
        .with_code(self.meta.code)
        .with_annotation(Annotation::primary(string.span).with_message("This rule string can be written as an array"))
        .with_note(
            "Array-style rules keep a consistent structure and avoid ambiguity when a rule value contains a `|`.",
        )
        .with_help("Replace the `|`-delimited string with an array of individual rules.");

        ctx.collector.propose(issue, |edits| {
            edits.push(TextEdit::replace(string.span, replacement.clone()));
        });
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_failure! {
        name = pipe_delimited_rule_string,
        rule = PreferArrayValidationRulesRule,
        count = 2,
        code = indoc! {r"
            <?php

            class StoreUserRequest
            {
                public function rules(): array
                {
                    return [
                        'name' => 'required|string|max:255',
                        'email' => 'required|email',
                    ];
                }
            }
        "}
    }

    test_lint_success! {
        name = array_rules,
        rule = PreferArrayValidationRulesRule,
        code = indoc! {r"
            <?php

            class StoreUserRequest
            {
                public function rules(): array
                {
                    return [
                        'name' => ['required', 'string'],
                    ];
                }
            }
        "}
    }

    test_lint_success! {
        name = single_rule_string,
        rule = PreferArrayValidationRulesRule,
        code = indoc! {r"
            <?php

            class StoreUserRequest
            {
                public function rules(): array
                {
                    return [
                        'name' => 'required',
                    ];
                }
            }
        "}
    }

    test_lint_success! {
        name = regex_rule_not_split,
        rule = PreferArrayValidationRulesRule,
        code = indoc! {r"
            <?php

            class StoreUserRequest
            {
                public function rules(): array
                {
                    return [
                        'slug' => 'required|regex:/^[a-z]+|[0-9]+$/',
                    ];
                }
            }
        "}
    }

    test_lint_success! {
        name = pipe_string_outside_rules_method,
        rule = PreferArrayValidationRulesRule,
        code = indoc! {r"
            <?php

            class Report
            {
                public function build(): array
                {
                    return [
                        'label' => 'left|right',
                    ];
                }
            }
        "}
    }

    test_lint_fix! {
        name = fix_pipe_string_to_array,
        rule = PreferArrayValidationRulesRule,
        code = indoc! {r"
            <?php

            class StoreUserRequest
            {
                public function rules(): array
                {
                    return [
                        'name' => 'required|string|max:255',
                    ];
                }
            }
        "},
        fixed = indoc! {r"
            <?php

            class StoreUserRequest
            {
                public function rules(): array
                {
                    return [
                        'name' => ['required', 'string', 'max:255'],
                    ];
                }
            }
        "}
    }
}
