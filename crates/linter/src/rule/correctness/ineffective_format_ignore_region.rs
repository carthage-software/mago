use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::ClassLikeMember;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Program;
use mago_syntax::ast::Statement;
use mago_syntax::walker::MutWalker;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

/// Markers that indicate the start of an unformatted region.
const IGNORE_START_MARKERS: [&str; 2] = ["@mago-format-ignore-start", "@mago-formatter-ignore-start"];

/// Markers that indicate the end of an unformatted region.
const IGNORE_END_MARKERS: [&str; 2] = ["@mago-format-ignore-end", "@mago-formatter-ignore-end"];

#[derive(Debug, Clone)]
pub struct IneffectiveFormatIgnoreRegionRule {
    meta: &'static RuleMeta,
    cfg: IneffectiveFormatIgnoreRegionConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct IneffectiveFormatIgnoreRegionConfig {
    pub level: Level,
}

impl Default for IneffectiveFormatIgnoreRegionConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for IneffectiveFormatIgnoreRegionConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for IneffectiveFormatIgnoreRegionRule {
    type Config = IneffectiveFormatIgnoreRegionConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Ineffective Format Ignore Region",
            code: "ineffective-format-ignore-region",
            description: indoc! {"
                Detects `@mago-format-ignore-start` markers that will have no effect.

                The formatter's ignore regions work at the statement level. When an
                ignore marker is placed inside an expression (like function call arguments,
                array elements, or other non-statement contexts), it will not affect
                the formatter's output.

                To effectively ignore a region, place the ignore markers between complete
                statements at the top level of a block or file.
            "},
            good_example: indoc! {r#"
                <?php

                // This works - markers are between statements
                // @mago-format-ignore-start
                $x = 1;  $y = 2;  // preserved as-is
                // @mago-format-ignore-end

                foo();
            "#},
            bad_example: indoc! {r#"
                <?php

                // This doesn't work - markers are inside a function call
                foo( // @mago-format-ignore-start
                    $x,
                    $y
                // @mago-format-ignore-end
                );
            "#},
            category: Category::Correctness,

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

        // Build ignore regions from comments
        let mut regions = build_ignore_regions(program);

        if regions.is_empty() {
            return;
        }

        // Walk the AST to mark regions as used when they contain statement starts
        let mut checker = IgnoreRegionChecker { regions: &mut regions };
        mago_syntax::walker::walk_program_mut(&mut checker, program, &mut ());

        // Report any regions that weren't used
        for region in regions {
            if !region.used {
                let issue = Issue::new(self.cfg.level(), "This format-ignore region has no effect.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(region.start_marker_span)
                            .with_message("This marker is inside an expression, not between statements."),
                    )
                    .with_note(
                        "Format-ignore regions only work at the statement level. \
                     Markers placed inside expressions (like function call arguments) will be ignored.",
                    )
                    .with_help("Move the markers to surround complete statements, not expressions within statements.");

                ctx.collector.report(issue);
            }
        }
    }
}

/// Represents an ignore region with its start marker span.
struct IgnoreRegion {
    /// The span of the start marker comment.
    start_marker_span: Span,
    /// Start offset of the region (beginning of the start comment).
    start: u32,
    /// End offset of the region (end of the end comment, or source length if no end).
    end: u32,
    /// Whether this region contains at least one statement start.
    used: bool,
}

/// Builds ignore regions from comments in the program.
fn build_ignore_regions(program: &Program<'_>) -> Vec<IgnoreRegion> {
    let mut regions = Vec::new();
    let mut current_start: Option<(Span, u32)> = None;
    let source_len = program.source_text.len() as u32;

    for trivia in program.trivia.iter() {
        if !trivia.kind.is_comment() {
            continue;
        }

        let has_start = IGNORE_START_MARKERS.iter().any(|m| trivia.value.contains(m));
        let has_end = IGNORE_END_MARKERS.iter().any(|m| trivia.value.contains(m));

        if has_start && current_start.is_none() {
            current_start = Some((trivia.span, trivia.span.start.offset));
        } else if let Some((start_span, start_offset)) = current_start
            && has_end
        {
            regions.push(IgnoreRegion {
                start_marker_span: start_span,
                start: start_offset,
                end: trivia.span.end.offset,
                used: false,
            });

            current_start = None;
        }
    }

    if let Some((start_span, start_offset)) = current_start {
        regions.push(IgnoreRegion { start_marker_span: start_span, start: start_offset, end: source_len, used: false });
    }

    regions
}

/// Walker context that tracks which ignore regions are used.
struct IgnoreRegionChecker<'a> {
    regions: &'a mut [IgnoreRegion],
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for IgnoreRegionChecker<'_> {
    fn walk_in_statement(&mut self, statement: &'ast Statement<'arena>, _context: &mut ()) {
        let stmt_start = statement.span().start.offset;

        // Mark any region that contains this statement's start as used
        for region in self.regions.iter_mut() {
            if stmt_start >= region.start && stmt_start < region.end {
                region.used = true;
            }
        }
    }

    fn walk_in_class_like_member(&mut self, member: &'ast ClassLikeMember<'arena>, _context: &mut ()) {
        let member_start = member.span().start.offset;

        // Mark any region that contains this member's start as used
        for region in self.regions.iter_mut() {
            if member_start >= region.start && member_start < region.end {
                region.used = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::test_lint_failure;
    use crate::test_lint_success;

    use super::*;

    // Valid uses - should NOT produce warnings

    test_lint_success! {
        name = valid_between_statements,
        rule = IneffectiveFormatIgnoreRegionRule,
        code = indoc! {r#"
            <?php

            $a = 1;
            // @mago-format-ignore-start
            $b = 2;  $c = 3;  // multiple on same line
            // @mago-format-ignore-end
            $d = 4;
        "#}
    }

    test_lint_success! {
        name = valid_multiple_statements_in_region,
        rule = IneffectiveFormatIgnoreRegionRule,
        code = indoc! {r#"
            <?php

            // @mago-format-ignore-start
            $a = 1;
            $b = 2;
            $c = 3;
            // @mago-format-ignore-end
        "#}
    }

    test_lint_success! {
        name = valid_no_end_marker,
        rule = IneffectiveFormatIgnoreRegionRule,
        code = indoc! {r#"
            <?php

            // @mago-format-ignore-start
            $a = 1;
            $b = 2;
        "#}
    }

    test_lint_success! {
        name = valid_in_function_body,
        rule = IneffectiveFormatIgnoreRegionRule,
        code = indoc! {r#"
            <?php

            function foo() {
                // @mago-format-ignore-start
                $a=1;  $b=2;
                // @mago-format-ignore-end
            }
        "#}
    }

    test_lint_success! {
        name = valid_in_class_body,
        rule = IneffectiveFormatIgnoreRegionRule,
        code = indoc! {r#"
            <?php

            class Foo {
                // @mago-format-ignore-start
                public const A = 1;
                public const B = 2;
                // @mago-format-ignore-end
            }
        "#}
    }

    test_lint_success! {
        name = valid_in_trait,
        rule = IneffectiveFormatIgnoreRegionRule,
        code = indoc! {r#"
            <?php

            trait MyTrait {
                // @mago-format-ignore-start
                public $a = 1;
                public $b = 2;
                // @mago-format-ignore-end
            }
        "#}
    }

    test_lint_success! {
        name = valid_in_interface,
        rule = IneffectiveFormatIgnoreRegionRule,
        code = indoc! {r#"
            <?php

            interface MyInterface {
                // @mago-format-ignore-start
                public const A = 1;
                public function foo();
                // @mago-format-ignore-end
            }
        "#}
    }

    test_lint_success! {
        name = valid_in_enum,
        rule = IneffectiveFormatIgnoreRegionRule,
        code = indoc! {r#"
            <?php

            enum Status: int {
                // @mago-format-ignore-start
                case A = 1;
                case B = 2;
                // @mago-format-ignore-end
            }
        "#}
    }

    test_lint_success! {
        name = valid_in_anonymous_class,
        rule = IneffectiveFormatIgnoreRegionRule,
        code = indoc! {r#"
            <?php

            $obj = new class {
                // @mago-format-ignore-start
                public const A = 1;
                public $prop = 2;
                // @mago-format-ignore-end
            };
        "#}
    }

    test_lint_success! {
        name = valid_mixed_members,
        rule = IneffectiveFormatIgnoreRegionRule,
        code = indoc! {r#"
            <?php

            class Foo {
                // @mago-format-ignore-start
                public const A = 1;
                public $prop = 2;
                public function foo() {}
                // @mago-format-ignore-end
            }
        "#}
    }

    test_lint_success! {
        name = no_markers_at_all,
        rule = IneffectiveFormatIgnoreRegionRule,
        code = indoc! {r#"
            <?php

            $a = 1;
            $b = 2;
        "#}
    }

    // Invalid uses - SHOULD produce warnings

    test_lint_failure! {
        name = invalid_inside_function_call_args,
        rule = IneffectiveFormatIgnoreRegionRule,
        count = 1,
        code = indoc! {r#"
            <?php

            foo( // @mago-format-ignore-start
                $x,
                $y
            // @mago-format-ignore-end
            );
        "#}
    }

    test_lint_failure! {
        name = invalid_inside_array,
        rule = IneffectiveFormatIgnoreRegionRule,
        count = 1,
        code = indoc! {r#"
            <?php

            $arr = [ // @mago-format-ignore-start
                1,
                2,
                3
            // @mago-format-ignore-end
            ];
        "#}
    }

    test_lint_failure! {
        name = invalid_inside_method_chain,
        rule = IneffectiveFormatIgnoreRegionRule,
        count = 1,
        code = indoc! {r#"
            <?php

            $result = $foo // @mago-format-ignore-start
                ->bar()
                ->baz()
            // @mago-format-ignore-end
            ;
        "#}
    }
}
