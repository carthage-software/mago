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
use mago_syntax::ast::Statement;
use mago_syntax::walker::MutWalker;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

/// Markers that indicate the next statement should be ignored.
const IGNORE_NEXT_MARKERS: [&str; 2] = ["@mago-format-ignore-next", "@mago-formatter-ignore-next"];

#[derive(Debug, Clone)]
pub struct IneffectiveFormatIgnoreNextRule {
    meta: &'static RuleMeta,
    cfg: IneffectiveFormatIgnoreNextConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct IneffectiveFormatIgnoreNextConfig {
    pub level: Level,
}

impl Default for IneffectiveFormatIgnoreNextConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for IneffectiveFormatIgnoreNextConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for IneffectiveFormatIgnoreNextRule {
    type Config = IneffectiveFormatIgnoreNextConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Ineffective Format Ignore Next",
            code: "ineffective-format-ignore-next",
            description: indoc! {"
                Detects `@mago-format-ignore-next` markers that will have no effect.

                The formatter's ignore-next marker works at the statement level. When a
                marker is placed inside an expression (like function call arguments,
                array elements, or other non-statement contexts), it will not affect
                the formatter's output.

                To effectively ignore the next statement, place the marker immediately
                before a complete statement at the top level of a block or file.
            "},
            good_example: indoc! {r#"
                <?php

                // This works - marker is before a statement
                // @mago-format-ignore-next
                const GRID = [
                  [1, 2, 3], [1, 2, ], [0,    0],
                ];

                foo();
            "#},
            bad_example: indoc! {r#"
                <?php

                // This doesn't work - marker is inside an array literal
                $arr = [ // @mago-format-ignore-next
                    1,
                    2,
                ];
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

        // Build ignore-next markers from comments
        let mut markers = build_ignore_next_markers(program);

        if markers.is_empty() {
            return;
        }

        // Walk the AST to mark markers as used when they have a statement following
        let mut checker = IgnoreNextChecker { markers: &mut markers };
        mago_syntax::walker::walk_program_mut(&mut checker, program, &mut ());

        // Report any markers that weren't used
        for marker in markers {
            if !marker.used {
                let issue = Issue::new(self.cfg.level(), "This format-ignore-next marker has no effect.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(marker.marker_span)
                            .with_message("This marker is inside an expression, not before a statement."),
                    )
                    .with_note(
                        "Format-ignore-next markers only work at the statement level. \
                     Markers placed inside expressions (like array literals) will be ignored.",
                    )
                    .with_help("Move the marker to appear immediately before a complete statement.");

                ctx.collector.report(issue);
            }
        }
    }
}

/// Represents an ignore-next marker with its span and whether it's been used.
struct IgnoreNextMarker {
    /// The span of the marker comment.
    marker_span: Span,
    /// Whether this marker has a statement immediately following it.
    used: bool,
}

/// Builds ignore-next markers from comments in the program.
fn build_ignore_next_markers(program: &mago_syntax::ast::Program<'_>) -> Vec<IgnoreNextMarker> {
    let mut markers = Vec::new();

    for trivia in program.trivia.iter() {
        if !trivia.kind.is_comment() {
            continue;
        }

        if IGNORE_NEXT_MARKERS.iter().any(|m| trivia.value.contains(m)) {
            markers.push(IgnoreNextMarker { marker_span: trivia.span, used: false });
        }
    }

    markers
}

struct IgnoreNextChecker<'a> {
    markers: &'a mut [IgnoreNextMarker],
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for IgnoreNextChecker<'_> {
    fn walk_in_statement(&mut self, statement: &'ast Statement<'arena>, _context: &mut ()) {
        let stmt_start = statement.span().start.offset;

        for marker in self.markers.iter_mut() {
            if !marker.used && stmt_start > marker.marker_span.end_offset() {
                marker.used = true;
                break;
            }
        }
    }

    fn walk_in_class_like_member(&mut self, member: &'ast ClassLikeMember<'arena>, _context: &mut ()) {
        let member_start = member.span().start.offset;

        for marker in self.markers.iter_mut() {
            if !marker.used && member_start > marker.marker_span.end_offset() {
                marker.used = true;
                break;
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

    test_lint_success! {
        name = valid_before_statement,
        rule = IneffectiveFormatIgnoreNextRule,
        code = indoc! {r#"
            <?php

            $a = 1;
            // @mago-format-ignore-next
            $b=2;  $c=3;
            $d = 4;
        "#}
    }

    test_lint_success! {
        name = valid_before_const,
        rule = IneffectiveFormatIgnoreNextRule,
        code = indoc! {r#"
            <?php

            // @mago-format-ignore-next
            const GRID = [
              [1, 2, 3], [1, 2, ], [0,    0],
            ];
        "#}
    }

    test_lint_success! {
        name = valid_in_function_body,
        rule = IneffectiveFormatIgnoreNextRule,
        code = indoc! {r#"
            <?php

            function foo() {
                // @mago-format-ignore-next
                $a=1;  $b=2;
            }
        "#}
    }

    test_lint_success! {
        name = valid_in_class_body,
        rule = IneffectiveFormatIgnoreNextRule,
        code = indoc! {r#"
            <?php

            class Foo {
                // @mago-format-ignore-next
                public const GRID = [
                  [1, 2, 3], [1, 2, ], [0,    0],
                ];
            }
        "#}
    }

    test_lint_success! {
        name = valid_before_property,
        rule = IneffectiveFormatIgnoreNextRule,
        code = indoc! {r#"
            <?php

            class Foo {
                // @mago-format-ignore-next
                public $prop = 123;
            }
        "#}
    }

    test_lint_success! {
        name = valid_before_method,
        rule = IneffectiveFormatIgnoreNextRule,
        code = indoc! {r#"
            <?php

            class Foo {
                // @mago-format-ignore-next
                public function foo() {}
            }
        "#}
    }

    test_lint_success! {
        name = valid_in_trait,
        rule = IneffectiveFormatIgnoreNextRule,
        code = indoc! {r#"
            <?php

            trait MyTrait {
                // @mago-format-ignore-next
                public $prop = 123;
            }
        "#}
    }

    test_lint_success! {
        name = valid_in_interface,
        rule = IneffectiveFormatIgnoreNextRule,
        code = indoc! {r#"
            <?php

            interface MyInterface {
                // @mago-format-ignore-next
                public function foo();
            }
        "#}
    }

    test_lint_success! {
        name = valid_in_enum,
        rule = IneffectiveFormatIgnoreNextRule,
        code = indoc! {r#"
            <?php

            enum Status: int {
                // @mago-format-ignore-next
                case Pending = 1;
            }
        "#}
    }

    test_lint_success! {
        name = valid_in_anonymous_class,
        rule = IneffectiveFormatIgnoreNextRule,
        code = indoc! {r#"
            <?php

            $obj = new class {
                // @mago-format-ignore-next
                public const A = 1;
            };
        "#}
    }

    test_lint_success! {
        name = valid_before_trait_use,
        rule = IneffectiveFormatIgnoreNextRule,
        code = indoc! {r#"
            <?php

            trait SomeTrait {}

            class Foo {
                // @mago-format-ignore-next
                use SomeTrait;
            }
        "#}
    }

    test_lint_success! {
        name = valid_multiple_markers,
        rule = IneffectiveFormatIgnoreNextRule,
        code = indoc! {r#"
            <?php

            // @mago-format-ignore-next
            $a=1;
            // @mago-format-ignore-next
            $b=2;
        "#}
    }

    test_lint_success! {
        name = no_markers_at_all,
        rule = IneffectiveFormatIgnoreNextRule,
        code = indoc! {r#"
            <?php

            $a = 1;
            $b = 2;
        "#}
    }

    test_lint_failure! {
        name = invalid_inside_array,
        rule = IneffectiveFormatIgnoreNextRule,
        count = 1,
        code = indoc! {r#"
            <?php

            $arr = [ // @mago-format-ignore-next
                1,
                2,
                3
            ];
        "#}
    }

    test_lint_failure! {
        name = invalid_inside_function_call,
        rule = IneffectiveFormatIgnoreNextRule,
        count = 1,
        code = indoc! {r#"
            <?php

            foo( // @mago-format-ignore-next
                $x,
                $y
            );
        "#}
    }

    test_lint_failure! {
        name = invalid_inside_method_chain,
        rule = IneffectiveFormatIgnoreNextRule,
        count = 1,
        code = indoc! {r#"
            <?php

            $result = $foo // @mago-format-ignore-next
                ->bar()
                ->baz();
        "#}
    }
}
