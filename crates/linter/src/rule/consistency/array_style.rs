use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct ArrayStyleRule {
    meta: &'static RuleMeta,
    cfg: ArrayStyleConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum ArrayStyleOption {
    Short,
    Long,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct ArrayStyleConfig {
    pub level: Level,
    pub style: ArrayStyleOption,
}

impl Default for ArrayStyleConfig {
    fn default() -> Self {
        Self { level: Level::Note, style: ArrayStyleOption::Short }
    }
}

impl Config for ArrayStyleConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ArrayStyleRule {
    type Config = ArrayStyleConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Array Style",
            code: "array-style",
            description: indoc! {"
                Suggests using the short array style `[..]` instead of the long array style `array(..)`,
                or vice versa, depending on the configuration. The short array style is more concise and
                is the preferred way to define arrays in PHP.
            "},
            good_example: indoc! {r"
                <?php

                // By default, `style` is 'short', so this snippet is valid:
                $arr = [1, 2, 3];
            "},
            bad_example: indoc! {r"
                <?php

                // By default, 'short' is enforced, so array(...) triggers a warning:
                $arr = array(1, 2, 3);
            "},
            category: Category::Consistency,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::LegacyArray, NodeKind::Array];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        match node {
            Node::LegacyArray(arr) if ArrayStyleOption::Short == self.cfg.style => {
                let issue = Issue::new(self.cfg.level(), "Short array style `[..]` is preferred over `array(..)`.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(arr.span())
                            .with_message("This array uses the long array style `array(..)`"),
                    )
                    .with_help("Use the short array style `[..]` instead.");

                ctx.collector.propose(issue, |edits| {
                    if arr.left_parenthesis.end == arr.right_parenthesis.start {
                        edits.push(TextEdit::replace(arr.array.span.join(arr.right_parenthesis), "[]"));
                    } else {
                        edits.push(TextEdit::replace(arr.array.span.join(arr.left_parenthesis), "["));
                        edits.push(TextEdit::replace(arr.right_parenthesis, "]"));
                    }
                });
            }
            Node::Array(arr) if ArrayStyleOption::Long == self.cfg.style && !is_destructuring_target(ctx) => {
                let issue = Issue::new(self.cfg.level(), "Long array style `array(..)` is preferred over `[..]`.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(arr.span()).with_message("This array uses the short array style `[..]`"),
                    )
                    .with_help("Use the long array style `array(..)` instead.");

                ctx.collector.propose(issue, |edits| {
                    edits.push(TextEdit::replace(arr.left_bracket, "array("));
                    edits.push(TextEdit::replace(arr.right_bracket, ")"));
                });
            }
            _ => {}
        }
    }
}

/// Whether the current `[..]` is the target of a destructuring assignment or a `foreach` value,
/// directly or nested inside another one: `[$a, [$b]] = $c;`, `list([$a]) = $c;`, `foreach ($x as [$a, $b])`.
///
/// There, `[..]` is a short list, not an array, and `array(..)` would not parse; the long
/// form is `list(..)`, which this rule does not enforce.
fn is_destructuring_target<A>(ctx: &LintContext<'_, '_, A>) -> bool
where
    A: Arena,
{
    let mut child_span = None;
    let mut depth = 0;
    while let Some(parent) = ctx.get_nth_parent(depth) {
        match parent {
            Node::Expression(_)
            | Node::Array(_)
            | Node::ArrayElement(_)
            | Node::ValueArrayElement(_)
            | Node::KeyValueArrayElement(_) => {
                child_span = Some(parent.span());
                depth += 1;
            }
            Node::Assignment(assignment) => {
                return child_span.is_some_and(|span| assignment.lhs.span() == span);
            }
            Node::List(_) | Node::ForeachValueTarget(_) | Node::ForeachKeyValueTarget(_) => return true,
            _ => return false,
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::ArrayStyleOption;
    use super::ArrayStyleRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = long_style_skips_destructuring_targets,
        rule = ArrayStyleRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.array_style.config.style = ArrayStyleOption::Long;
        },
        code = indoc! {r"
            <?php

            [$a, $b] = $pair;
            [$c, [$d, $e]] = $nested;
            ['x' => $f, 'y' => $g] = $point;
            foreach ($pairs as [$h, $i]) {}
            foreach ($pairs as $key => [$j, $k]) {}
            list([$m, $n]) = $nested;
            foreach ($pairs as list([$o])) {}
            $l = array(1, 2);
        "}
    }

    test_lint_failure! {
        name = long_style_flags_arrays_next_to_destructuring,
        rule = ArrayStyleRule,
        count = 5,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.array_style.config.style = ArrayStyleOption::Long;
        },
        code = indoc! {r"
            <?php

            $a = [1, 2];
            [$b, $c] = [3, 4];
            [$d] = [[5]];
            $e[[6][0]] = 7;
        "}
    }
}
