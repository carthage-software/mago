use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasPosition;
use mago_span::HasSpan;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_text_edit::Safety;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoFullyQualifiedGlobalFunctionRule {
    meta: &'static RuleMeta,
    cfg: NoFullyQualifiedGlobalFunctionConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoFullyQualifiedGlobalFunctionConfig {
    pub level: Level,
    pub namespaced: bool,
}

impl Default for NoFullyQualifiedGlobalFunctionConfig {
    fn default() -> Self {
        Self { level: Level::Help, namespaced: false }
    }
}

impl Config for NoFullyQualifiedGlobalFunctionConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoFullyQualifiedGlobalFunctionRule {
    type Config = NoFullyQualifiedGlobalFunctionConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Fully Qualified Global Function",
            code: "no-fully-qualified-global-function",
            description: indoc! {"
                Disallows fully-qualified references to global functions that could be shortened or imported.

                Instead of using the backslash prefix (e.g., `\\strlen()`),
                prefer an explicit `use function` import statement. This improves
                readability and keeps imports centralized at the top of the file.
            "},
            good_example: indoc! {r#"
                <?php

                namespace App;

                use function strlen;

                $length = strlen("hello");
            "#},
            bad_example: indoc! {r#"
                <?php

                namespace App;

                $length = \strlen("hello");
            "#},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionCall, NodeKind::FunctionPartialApplication];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let identifier = match node {
            Node::FunctionCall(call) => {
                let Expression::Identifier(identifier) = call.function else {
                    return;
                };
                identifier
            }
            Node::FunctionPartialApplication(application) => {
                let Expression::Identifier(identifier) = application.function else {
                    return;
                };
                identifier
            }
            _ => return,
        };

        let (function_name_bytes, is_bare_call): (&[u8], bool) = if identifier.is_fully_qualified() {
            (mago_bytes::trim_start_byte(identifier.value(), b'\\'), false)
        } else if self.cfg.namespaced
            && !identifier.value().contains(&b'\\')
            && ctx.is_name_imported(&identifier.position())
        {
            let resolved = ctx.lookup_name(&identifier.position());
            let stripped = mago_bytes::trim_start_byte(resolved, b'\\');
            if !stripped.contains(&b'\\') {
                return;
            }

            (stripped, true)
        } else {
            return;
        };

        let short_name_bytes = function_name_bytes.rsplit(|&b| b == b'\\').next().unwrap_or(function_name_bytes);
        let fqn_span = identifier.span();
        let function_name = mago_bytes::BytesDisplay(function_name_bytes);
        let short_name = mago_bytes::BytesDisplay(short_name_bytes);

        let namespace_part: Option<&[u8]> = if self.cfg.namespaced {
            memchr::memrchr(b'\\', function_name_bytes).map(|i| &function_name_bytes[..i]).filter(|ns| !ns.is_empty())
        } else {
            None
        };

        let (resolution, replacement, use_statement_text) = match namespace_part {
            Some(ns_bytes) => {
                let ns_display = mago_bytes::BytesDisplay(ns_bytes);
                match ctx.import_name(ns_bytes) {
                    Some(res) => {
                        let replacement = format!("{}\\{short_name}", res.local_name);
                        let use_text = format!("use {ns_display};");
                        (Some(res), Some(replacement), use_text)
                    }
                    None => (None, None, format!("use {ns_display};")),
                }
            }
            None => match ctx.import_function(function_name_bytes) {
                Some(res) => {
                    let replacement = res.local_name.to_string();
                    let use_text = format!("use function {function_name};");
                    (Some(res), Some(replacement), use_text)
                }
                None => (None, None, format!("use function {function_name};")),
            },
        };

        let short_name_str = short_name.to_string();
        let (title, help) = match (is_bare_call, &resolution, replacement.as_deref()) {
            (true, Some(res), Some(rep)) if res.is_already_available() => (
                "`use function` import can be migrated to a namespace import.",
                format!(
                    "`{function_name}` is already reachable as `{rep}`; replace the call with it and drop the `use function` import."
                ),
            ),
            (true, _, _) => (
                "`use function` import can be migrated to a namespace import.",
                format!(
                    "Add `{use_statement_text}` and call `{}(...)`, then drop the `use function` import.",
                    replacement.as_deref().unwrap_or(&short_name_str)
                ),
            ),
            (false, Some(res), Some(rep)) if res.is_already_available() && rep != short_name_str => (
                "Fully-qualified function call can be replaced with an existing alias.",
                format!("`{function_name}` is already reachable as `{rep}`; replace the call with it."),
            ),
            (false, Some(res), _) if res.is_already_available() => (
                "Fully-qualified function call is already in scope.",
                format!("`{function_name}` is already reachable as `{short_name}`; drop the leading `\\`."),
            ),
            (false, _, _) => (
                "Fully-qualified function call detected.",
                format!(
                    "Add `{use_statement_text}` and call `{}(...)` directly.",
                    replacement.as_deref().unwrap_or(&short_name_str)
                ),
            ),
        };

        let annotation_message = if is_bare_call {
            format!("This call resolves to `\\{function_name}` via a `use function` import")
        } else {
            format!("The call to `\\{function_name}` uses a fully-qualified name")
        };

        let issue = Issue::new(self.cfg.level, title)
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(fqn_span).with_message(annotation_message))
            .with_note(if is_bare_call {
                "Grouping helpers under a single namespace import keeps related calls (`Str\\length`, `Str\\trim`, …) visibly clustered instead of scattered across `use function` lines."
            } else {
                "Fully-qualified function calls bypass the import system, making it harder to see which global functions a file depends on."
            })
            .with_help(help);

        let orphaned_use_span = if is_bare_call { ctx.sole_function_import_use_span(identifier.value()) } else { None };

        match (resolution, replacement) {
            (Some(resolution), Some(replacement)) => {
                ctx.collector.propose(issue, |edits| {
                    edits.push(TextEdit::replace(fqn_span, replacement));
                    if let Some(use_edit) = resolution.use_statement_edit {
                        edits.push(use_edit.with_safety(Safety::Safe));
                    }
                    if let Some(span) = orphaned_use_span {
                        edits.push(TextEdit::delete(span).with_safety(Safety::Safe));
                    }
                });
            }
            _ => {
                ctx.collector.report(issue);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoFullyQualifiedGlobalFunctionRule;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_success! {
        name = imported_function_is_not_flagged,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use function strlen;

            $length = strlen("hello");
        "#}
    }

    test_lint_success! {
        name = unqualified_function_is_not_flagged,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $length = strlen("hello");
        "#}
    }

    test_lint_fix! {
        name = global_scope_fq_function_drops_leading_slash,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            $length = \strlen("hello");
        "#},
        fixed = indoc! {r#"
            <?php

            $length = strlen("hello");
        "#}
    }

    test_lint_fix! {
        name = global_scope_namespaced_function_is_imported,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            $r = \App\Support\camel_case("foo_bar");
        "#},
        fixed = indoc! {r#"
            <?php

            use function App\Support\camel_case;

            $r = camel_case("foo_bar");
        "#}
    }

    test_lint_failure! {
        name = fq_function_call_in_namespace,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $length = \strlen("hello");
        "#}
    }

    test_lint_failure! {
        name = fq_function_partial_application_in_namespace,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $fn = \strlen(...);
        "#}
    }

    test_lint_fix! {
        name = fix_fq_function_adds_use_function,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $length = \strlen("hello");
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use function strlen;

            $length = strlen("hello");
        "#}
    }

    test_lint_fix! {
        name = fix_fq_function_appends_after_existing_use_function,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use function array_map;

            $length = \strlen("hello");
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use function array_map;
            use function strlen;

            $length = strlen("hello");
        "#}
    }

    test_lint_fix! {
        name = fix_three_fq_functions_in_one_pass,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $a = \strlen("a");
            $b = \strtoupper("b");
            $c = \trim("c");
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use function strlen;

            use function strtoupper;

            use function trim;

            $a = strlen("a");
            $b = strtoupper("b");
            $c = trim("c");
        "#}
    }

    test_lint_fix! {
        name = fix_many_calls_to_same_fq_function_one_use,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $a = \strlen("a");
            $b = \strlen("b");
            $c = \strlen("c");
            $d = \strlen("d");
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use function strlen;

            $a = strlen("a");
            $b = strlen("b");
            $c = strlen("c");
            $d = strlen("d");
        "#}
    }

    test_lint_failure! {
        name = fix_declined_when_short_name_conflicts_with_local_function,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            function strlen(string $s): int { return 0; }

            $a = \strlen("a");
        "#}
    }

    test_lint_failure! {
        name = fix_declined_when_short_name_conflicts_with_existing_use_function,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use function Other\strlen;

            $a = \strlen("a");
        "#}
    }

    test_lint_fix! {
        name = fix_fq_function_appends_after_last_existing_use_function,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use function array_map;
            use function array_filter;
            use function array_reduce;

            $length = \strlen("hello");
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use function array_map;
            use function array_filter;
            use function array_reduce;
            use function strlen;

            $length = strlen("hello");
        "#}
    }

    test_lint_fix! {
        name = fix_fq_function_in_multiple_braced_namespaces_independent,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace A {
                $x = \strlen("a");
            }

            namespace B {
                $y = \strlen("b");
            }
        "#},
        fixed = indoc! {r#"
            <?php

            namespace A {

            use function strlen;
                $x = strlen("a");
            }

            namespace B {

            use function strlen;
                $y = strlen("b");
            }
        "#}
    }

    test_lint_success! {
        name = fq_function_via_existing_aliased_use_is_not_flagged_after_rewrite,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use function strlen as length;

            $n = length("hello");
        "#}
    }

    test_lint_failure! {
        name = fq_function_with_existing_alias_for_different_name_still_flagged_but_conflict_blocks_fix,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use function Other\something as strlen;

            $n = \strlen("hello");
        "#}
    }

    test_lint_success! {
        name = short_call_in_global_is_not_flagged,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            $n = strlen("x");
        "#}
    }

    test_lint_failure! {
        name = fq_function_partial_application_in_namespace_flagged,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $fn = \strlen(...);
        "#}
    }

    test_lint_fix! {
        name = fix_fq_function_partial_application_adds_use,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $fn = \strlen(...);
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use function strlen;

            $fn = strlen(...);
        "#}
    }

    test_lint_success! {
        name = fq_function_in_nested_sub_namespace_with_matching_current_is_handled,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App\Sub;

            use function Other\helper;

            $x = helper();
        "#}
    }

    test_lint_fix! {
        name = namespaced_fix_imports_namespace_and_prefixes_call,
        rule = NoFullyQualifiedGlobalFunctionRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.no_fully_qualified_global_function.config.namespaced = true;
        },
        code = indoc! {r#"
            <?php

            namespace App;

            $a = \Psl\Str\length($s);
            $b = \Psl\Str\Byte\length($s);
            $c = \Psl\Str\Grapheme\length($s);
            $a = \Psl\Str\length($s);
            $b = \Psl\Str\Byte\length($s);
            $c = \Psl\Str\Grapheme\length($s);
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use Psl\Str;

            use Psl\Str\Byte;

            use Psl\Str\Grapheme;

            $a = Str\length($s);
            $b = Byte\length($s);
            $c = Grapheme\length($s);
            $a = Str\length($s);
            $b = Byte\length($s);
            $c = Grapheme\length($s);
        "#}
    }

    test_lint_fix! {
        name = namespaced_fix_uses_function_import_when_no_namespace,
        rule = NoFullyQualifiedGlobalFunctionRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.no_fully_qualified_global_function.config.namespaced = true;
        },
        code = indoc! {r#"
            <?php

            namespace App;

            $n = \strlen("x");
            $n = \strlen("x");
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use function strlen;

            $n = strlen("x");
            $n = strlen("x");
        "#}
    }

    test_lint_fix! {
        name = namespaced_fix_reuses_existing_namespace_import,
        rule = NoFullyQualifiedGlobalFunctionRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.no_fully_qualified_global_function.config.namespaced = true;
        },
        code = indoc! {r#"
            <?php

            namespace App;

            use Psl\Str;

            $n = \Psl\Str\length($s);
            $n = \Psl\Str\length($s);
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use Psl\Str;

            $n = Str\length($s);
            $n = Str\length($s);
        "#}
    }

    test_lint_fix! {
        name = namespaced_fix_uses_existing_aliased_namespace_import,
        rule = NoFullyQualifiedGlobalFunctionRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.no_fully_qualified_global_function.config.namespaced = true;
        },
        code = indoc! {r#"
            <?php

            namespace App;

            use Psl\Str as S;

            $n = \Psl\Str\length($s);
            $n = \Psl\Str\length($s);
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use Psl\Str as S;

            $n = S\length($s);
            $n = S\length($s);
        "#}
    }

    test_lint_success! {
        name = namespaced_unqualified_namespace_prefixed_call_is_not_flagged,
        rule = NoFullyQualifiedGlobalFunctionRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.no_fully_qualified_global_function.config.namespaced = true;
        },
        code = indoc! {r#"
            <?php

            namespace App;

            use Psl\Str;

            $n = Str\length($s);
        "#}
    }

    test_lint_fix! {
        name = namespaced_fix_migrates_use_function_to_namespace_import,
        rule = NoFullyQualifiedGlobalFunctionRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.no_fully_qualified_global_function.config.namespaced = true;
        },
        code = indoc! {r#"
            <?php

            namespace App;

            use function Psl\invariant;

            invariant($x, 'msg');
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;


            use Psl;

            Psl\invariant($x, 'msg');
        "#}
    }

    test_lint_fix! {
        name = namespaced_fix_migrates_use_function_with_nested_namespace,
        rule = NoFullyQualifiedGlobalFunctionRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.no_fully_qualified_global_function.config.namespaced = true;
        },
        code = indoc! {r#"
            <?php

            namespace App;

            use function Psl\Str\length;

            $n = length($s);
            $m = length($t);
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;


            use Psl\Str;

            $n = Str\length($s);
            $m = Str\length($t);
        "#}
    }

    test_lint_fix! {
        name = namespaced_fix_migrates_use_function_reuses_existing_namespace_import,
        rule = NoFullyQualifiedGlobalFunctionRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.no_fully_qualified_global_function.config.namespaced = true;
        },
        code = indoc! {r#"
            <?php

            namespace App;

            use Psl\Str;
            use function Psl\Str\length;

            $n = length($s);
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use Psl\Str;


            $n = Str\length($s);
        "#}
    }

    test_lint_success! {
        name = namespaced_bare_use_function_for_global_function_is_not_migrated,
        rule = NoFullyQualifiedGlobalFunctionRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.no_fully_qualified_global_function.config.namespaced = true;
        },
        code = indoc! {r#"
            <?php

            namespace App;

            use function strlen;

            $n = strlen("x");
        "#}
    }

    test_lint_success! {
        name = unnamespaced_bare_use_function_call_is_not_flagged,
        rule = NoFullyQualifiedGlobalFunctionRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use function Psl\invariant;

            invariant($x, 'msg');
        "#}
    }
}
