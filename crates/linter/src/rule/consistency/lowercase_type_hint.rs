use indoc::indoc;
use mago_allocator::Arena;
use mago_text_edit::TextEdit;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Hint;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct LowercaseTypeHintRule {
    meta: &'static RuleMeta,
    cfg: LowercaseTypeHintConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct LowercaseTypeHintConfig {
    pub level: Level,
}

impl Default for LowercaseTypeHintConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for LowercaseTypeHintConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for LowercaseTypeHintRule {
    type Config = LowercaseTypeHintConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Lowercase Type Hint",
            code: "lowercase-type-hint",
            description: indoc! {"
                Enforces that PHP type hints (like `void`, `bool`, `int`, `float`, etc.) be written
                in lowercase. Using uppercase or mixed case is discouraged for consistency
                and readability.
            "},
            good_example: indoc! {r"
                <?php

                function example(int $param): void {
                    return;
                }
            "},
            bad_example: indoc! {r"
                <?php

                function example(Int $param): VOID {
                    return;
                }
            "},
            category: Category::Consistency,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Hint];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Hint(hint) = node else {
            return;
        };

        let (Hint::Void(identifier)
        | Hint::Never(identifier)
        | Hint::Float(identifier)
        | Hint::Bool(identifier)
        | Hint::Integer(identifier)
        | Hint::String(identifier)
        | Hint::Object(identifier)
        | Hint::Mixed(identifier)
        | Hint::Iterable(identifier)) = hint
        else {
            return;
        };

        if identifier.value.iter().all(|&b| !b.is_ascii_alphabetic() || b.is_ascii_lowercase()) {
            return; // Already in lowercase, no issue to report
        }

        let lowercase = identifier.value.to_ascii_lowercase();
        let Some(lowercase_str) = std::str::from_utf8(&lowercase).ok() else { return };
        let identifier_display = mago_bytes::BytesDisplay(identifier.value);

        let issue = Issue::new(self.cfg.level(), format!("Type hint `{identifier_display}` should be in lowercase."))
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(identifier.span()))
            .with_help(format!("Consider using `{lowercase_str}` instead of `{identifier_display}`."));

        let lowercase_owned = lowercase_str.to_owned();
        ctx.collector.propose(issue, |edits| {
            edits.push(TextEdit::replace(identifier.span, lowercase_owned));
        });
    }
}
