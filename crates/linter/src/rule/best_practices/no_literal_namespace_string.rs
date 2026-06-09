use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
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
pub struct NoLiteralNamespaceStringRule {
    meta: &'static RuleMeta,
    cfg: NoLiteralNamespaceStringConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoLiteralNamespaceStringConfig {
    pub level: Level,
}

impl Default for NoLiteralNamespaceStringConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoLiteralNamespaceStringConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoLiteralNamespaceStringRule {
    type Config = NoLiteralNamespaceStringConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Literal Namespace String",
            code: "no-literal-namespace-string",
            description: indoc! {"
                Flags hardcoded fully qualified class name strings. Use `::class` notation
                instead for better IDE support, refactoring safety, and static analysis.
            "},
            good_example: indoc! {r"
                <?php

                $className = \App\Models\User::class;
            "},
            bad_example: indoc! {r"
                <?php

                $className = 'App\Models\User';
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::LiteralString];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::LiteralString(literal) = node else {
            return;
        };

        let Some(value) = literal.value else {
            return;
        };

        if !looks_like_class_name(value) {
            return;
        }

        ctx.collector.report(
            Issue::new(self.cfg.level(), "Use `::class` notation instead of a hardcoded namespace string.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(literal.span).with_message("Hardcoded namespace string"))
                .with_help(
                    "Replace this string with `ClassName::class` for better IDE support and refactoring safety.",
                ),
        );
    }
}

/// Checks if a string value looks like a fully qualified class name.
///
/// Matches patterns like `Vendor\Module` or `Vendor\Module\ClassName` with at least 2 segments,
/// each starting with an uppercase letter.
fn looks_like_class_name(value: &[u8]) -> bool {
    let normalized: Vec<u8>;
    let s: &[u8] = if memchr::memmem::find(value, b"\\\\").is_some() {
        normalized = collapse_double_backslash(value);
        &normalized
    } else {
        value
    };

    let s = s.strip_prefix(b"\\").unwrap_or(s);

    if s.ends_with(b"\\") {
        return false;
    }

    let segments: Vec<&[u8]> = s.split(|&b| b == b'\\').collect();

    if segments.len() < 2 {
        return false;
    }

    segments.iter().all(|seg| {
        !seg.is_empty() && seg[0].is_ascii_uppercase() && seg.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_')
    })
}

fn collapse_double_backslash(value: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(value.len());
    let mut i = 0;
    while i < value.len() {
        if i + 1 < value.len() && value[i] == b'\\' && value[i + 1] == b'\\' {
            out.push(b'\\');
            i += 2;
        } else {
            out.push(value[i]);
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = normal_string,
        rule = NoLiteralNamespaceStringRule,
        code = r#"
            <?php

            $name = 'hello world';
        "#
    }

    test_lint_failure! {
        name = two_segment_namespace_string,
        rule = NoLiteralNamespaceStringRule,
        code = r#"
            <?php

            $name = 'Vendor\Module';
        "#
    }

    test_lint_failure! {
        name = three_segment_namespace_string,
        rule = NoLiteralNamespaceStringRule,
        code = r#"
            <?php

            $className = 'App\Models\User';
        "#
    }

    test_lint_failure! {
        name = double_backslash_namespace_string,
        rule = NoLiteralNamespaceStringRule,
        code = r#"
            <?php

            $className = 'App\\Models\\User';
        "#
    }
}
