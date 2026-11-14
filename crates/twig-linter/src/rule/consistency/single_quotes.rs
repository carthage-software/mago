use indoc::indoc;
use mago_allocator::Arena;
use mago_bytes::BytesDisplay;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_text_edit::TextEdit;
use mago_twig_syntax::cst::Node;
use mago_twig_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct SingleQuotesRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: SingleQuotesConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct SingleQuotesConfig {
    pub level: Level,
}

impl Default for SingleQuotesConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for SingleQuotesConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for SingleQuotesRule {
    type Config = SingleQuotesConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Single-Quoted Strings",
            code: "twig-single-quotes",
            description: indoc! {r"
                Prefers single quotes for plain strings: strings without escapes, apostrophes, `#`, or line
                breaks.

                Other strings stay unchanged, because new quotes would change their meaning.
            "},
            good_example: indoc! {r"
                {{ 'title' }}
            "},
            bad_example: indoc! {r#"
                {{ "title" }}
            "#},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::StringLiteral];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::StringLiteral(string) = node else {
            return;
        };

        if matches!(ctx.get_parent(), Some(Node::TypeDeclaration(_)))
            || string.raw.len() < 2
            || string.raw.first() != Some(&b'"')
            || string.raw.last() != Some(&b'"')
        {
            return;
        }

        let content = &string.raw[1..string.raw.len() - 1];
        if content.iter().any(|byte| matches!(byte, b'\'' | b'\\' | b'#' | b'\r' | b'\n')) {
            return;
        }

        ctx.collector.propose(
            Issue::new(self.cfg.level, "Use single quotes for this plain string.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(string.span).with_message("Double-quoted plain string"))
                .with_note(
                    "In Twig, double quotes allow `#{...}` interpolation. Single quotes mark a plain string, and the Twig coding standard prefers them.",
                )
                .with_help(format!("Write `'{}'` instead.", BytesDisplay(content))),
            |edits| {
                edits.push(TextEdit::replace(string.start_offset()..string.start_offset() + 1, "'"));
                edits.push(TextEdit::replace(string.end_offset() - 1..string.end_offset(), "'"));
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_success! {
        name = valid_template_1,
        rule = SingleQuotesRule,
        code = r"{{ 'title' }}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = SingleQuotesRule,
        code = r#"{{ "it's" }}"#,
    }

    test_lint_success! {
        name = valid_template_3,
        rule = SingleQuotesRule,
        code = r#"{{ "line\nnext" }}"#,
    }

    test_lint_success! {
        name = valid_template_4,
        rule = SingleQuotesRule,
        code = r##"{{ "#{value}" }}"##,
    }

    test_lint_success! {
        name = valid_template_5,
        rule = SingleQuotesRule,
        code = r##"{{ "#tag" }}"##,
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = SingleQuotesRule,
        count = 1,
        code = r#"{{ "title" }}"#,
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = SingleQuotesRule,
        count = 1,
        code = r#"{{ "" }}"#,
    }

    test_lint_failure! {
        name = invalid_template_3,
        rule = SingleQuotesRule,
        count = 2,
        code = r#"{{ {"title": "value"} }}"#,
    }

    test_lint_fix! {
        name = double_quotes_become_single_quotes,
        rule = SingleQuotesRule,
        code = r#"{{ "title" }}"#,
        fixed = r"{{ 'title' }}",
    }

    test_lint_success! {
        name = type_declarations_are_left_alone,
        rule = SingleQuotesRule,
        code = r#"{% types {name: "string"} %}"#,
    }
}
