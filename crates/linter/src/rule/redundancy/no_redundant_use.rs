use mago_allocator::Arena;
use std::collections::HashMap;
use std::collections::HashSet;

use indoc::indoc;
use schemars::JsonSchema;

use mago_database::file::HasFileId;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Inline;
use mago_syntax::ast::MixedUseItemList;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Program;
use mago_syntax::ast::Statement;
use mago_syntax::ast::UseItem;
use mago_syntax::ast::UseItems;
use mago_text_edit::TextEdit;
use mago_word::starts_with_ignore_case;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;
use mago_bytes::BytesDisplay;
use mago_bytes::trim_start_byte;

#[derive(Debug, Clone)]
pub struct NoRedundantUseRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantUseConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoRedundantUseConfig {
    pub level: Level,
}

impl Default for NoRedundantUseConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoRedundantUseConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantUseRule {
    type Config = NoRedundantUseConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Use",
            code: "no-redundant-use",
            description: indoc! {"
                Detects `use` statements that import items that are never used or are redundant
                because they import from the same namespace.
            "},
            good_example: indoc! {r"
                <?php
                namespace App;

                use App\Helpers\ArrayHelper;

                $result = ArrayHelper::combine([]);
            "},
            bad_example: indoc! {r"
                <?php
                namespace App;

                use App\Helpers\ArrayHelper;
                use App\Helpers\StringHelper; // StringHelper is not used.

                $result = ArrayHelper::combine([]);
            "},
            category: Category::Redundancy,
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

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Program(program) = node else { return };

        let mut check_inline_mentions = false;

        // If `tempest` integration is enabled, and this file ends with `.view.php`,
        // check inline mentions as well.
        if ctx.registry.is_integration_enabled(Integration::Tempest) {
            let path_str = ctx.source_file.path.as_ref().and_then(|p| p.to_str());
            let ends_with_view = match path_str {
                Some(p) => p.ends_with(".view.php"),
                None => ctx.source_file.name.as_ref().ends_with(b".view.php"),
            };
            if ends_with_view {
                check_inline_mentions = true;
            }
        }

        let use_declarations = utils::collect_use_declarations(program);
        if use_declarations.is_empty() {
            return;
        }

        let used_fqns = utils::build_used_fqn_set(ctx, &use_declarations);
        let docblocks = utils::get_docblocks(program);
        let inline_contents =
            if check_inline_mentions { utils::get_inline_contents(program) } else { Vec::with_capacity(0) };

        // First, check for same-namespace imports (redundant even if used)
        let mut same_namespace_spans: HashSet<Span> = HashSet::new();
        for decl in &use_declarations {
            if utils::is_same_namespace_import(decl) {
                let alias = utils::get_alias(decl.item);
                let Statement::Use(use_stmt) = decl.parent_stmt else { continue };
                same_namespace_spans.insert(decl.item.span());

                let alias_display = BytesDisplay(alias.as_bytes());
                let message = match &decl.namespace {
                    Some(ns) => format!(
                        "Redundant import: `{alias_display}` is already in the current namespace `{}`.",
                        BytesDisplay(ns.as_bytes())
                    ),
                    None => {
                        format!("Redundant import: `{alias_display}` is already available in the root namespace.")
                    }
                };

                let issue = Issue::new(self.cfg.level(), message)
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(decl.item.name.span())
                            .with_message(format!("`{alias_display}` does not need to be imported.")),
                    )
                    .with_annotation(
                        Annotation::secondary(use_stmt.r#use.span()).with_message("Redundant `use` statement."),
                    )
                    .with_help("Remove the import; the symbol is already accessible without it.");

                ctx.collector.propose(issue, |edits| {
                    if let Some(range) =
                        utils::calculate_delete_range_for_item(decl.parent_stmt, decl.item, ctx.source_file.file_id())
                    {
                        edits.push(TextEdit::delete(range));
                    }
                });
            }
        }

        let grouped_by_parent = use_declarations.into_iter().fold(HashMap::new(), |mut acc, decl| {
            acc.entry(decl.parent_stmt.span()).or_insert_with(Vec::new).push(decl);
            acc
        });

        for (_, mut decls) in grouped_by_parent {
            decls.retain(|d| !same_namespace_spans.contains(&d.item.span()));
            if decls.is_empty() {
                continue;
            }

            let total_items = decls.len();
            let unused_items: Vec<_> = decls
                .iter()
                .filter(|decl| !utils::is_item_used(decl, &used_fqns, &docblocks, &inline_contents))
                .collect();

            if unused_items.is_empty() {
                continue;
            }

            let parent_stmt = unused_items[0].parent_stmt;
            let Statement::Use(use_stmt) = parent_stmt else { continue };

            if unused_items.len() == total_items {
                if total_items == 1 {
                    let unused_decl = unused_items[0];
                    let alias = utils::get_alias(unused_decl.item);
                    let alias_display = BytesDisplay(alias.as_bytes());
                    let issue = Issue::new(self.cfg.level(), format!("Unused import: `{alias_display}`."))
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(unused_decl.item.name.span())
                                .with_message(format!("`{alias_display}` is imported but never used.")),
                        )
                        .with_annotation(
                            Annotation::secondary(use_stmt.r#use.span()).with_message("Unused `use` statement."),
                        )
                        .with_help("Remove the entire `use` statement.");

                    ctx.collector.propose(issue, |edits| {
                        edits.push(TextEdit::delete(parent_stmt.span()));
                    });
                } else {
                    let issue = Issue::new(self.cfg.level(), "Redundant `use` statement.")
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(parent_stmt.span())
                                .with_message("All symbols imported here are unused."),
                        )
                        .with_help("Remove the entire `use` statement.");

                    ctx.collector.propose(issue, |edits| {
                        edits.push(TextEdit::delete(parent_stmt.span()));
                    });
                }
            } else {
                let mut issue = Issue::new(self.cfg.level(), "Unused symbols in `use` statement.")
                    .with_code(self.meta.code)
                    .with_help("Remove the unused symbols from the import list.")
                    .with_annotation(
                        Annotation::secondary(use_stmt.r#use.span()).with_message("...in this `use` statement."),
                    );

                for unused_decl in &unused_items {
                    let alias = utils::get_alias(unused_decl.item);
                    let alias_display = BytesDisplay(alias.as_bytes());
                    issue = issue.with_annotation(
                        Annotation::primary(unused_decl.item.span())
                            .with_message(format!("`{alias_display}` is imported but never used.")),
                    );
                }

                ctx.collector.propose(issue, |edits| {
                    for unused_decl in unused_items.iter().rev() {
                        if let Some(delete_range) = utils::calculate_delete_range_for_item(
                            parent_stmt,
                            unused_decl.item,
                            ctx.source_file.file_id(),
                        ) {
                            edits.push(TextEdit::delete(delete_range));
                        }
                    }
                });
            }
        }
    }
}

mod utils {
    use foldhash::HashSet;
    use mago_allocator::Arena;
    use mago_database::file::FileId;
    use mago_span::Span;
    use mago_syntax::walker::MutWalker;
    use mago_word::Word;
    use mago_word::WordSet;
    use mago_word::concat_word;
    use mago_word::word;

    use super::HasSpan;
    use super::Inline;
    use super::LintContext;
    use super::MixedUseItemList;
    use super::Program;
    use super::Statement;
    use super::UseItem;
    use super::UseItems;
    use super::starts_with_ignore_case;
    use super::trim_start_byte;

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub(super) enum ImportType {
        ClassOrNamespace,
        Function,
        Constant,
    }

    #[derive(Debug, Clone)]
    pub(super) struct UseDeclaration<'ast> {
        pub parent_stmt: &'ast Statement<'ast>,
        pub item: &'ast UseItem<'ast>,
        pub import_type: ImportType,
        pub fqn: Word,
        pub namespace: Option<Word>,
    }

    pub(super) fn collect_use_declarations<'ast>(program: &'ast Program<'ast>) -> Vec<UseDeclaration<'ast>> {
        let mut declarations = Vec::new();
        for stmt in &program.statements {
            if let Statement::Namespace(ns) = stmt {
                let namespace = ns.name.as_ref().map(|n| word(n.value()));
                for ns_stmt in ns.statements() {
                    collect_from_statement(ns_stmt, namespace, &mut declarations);
                }
            } else {
                collect_from_statement(stmt, None, &mut declarations);
            }
        }
        declarations
    }

    fn collect_from_statement<'ast>(
        stmt: &'ast Statement<'ast>,
        namespace: Option<Word>,
        declarations: &mut Vec<UseDeclaration<'ast>>,
    ) {
        if let Statement::Use(use_stmt) = stmt {
            match &use_stmt.items {
                UseItems::Sequence(s) => {
                    let import_type = ImportType::ClassOrNamespace;
                    for item in s.items.nodes {
                        declarations.push(UseDeclaration {
                            parent_stmt: stmt,
                            item,
                            import_type,
                            fqn: word(trim_start_byte(item.name.value(), b'\\')),
                            namespace,
                        });
                    }
                }
                UseItems::TypedSequence(s) => {
                    let import_type = if s.r#type.is_function() { ImportType::Function } else { ImportType::Constant };
                    for item in s.items.nodes {
                        declarations.push(UseDeclaration {
                            parent_stmt: stmt,
                            item,
                            import_type,
                            fqn: word(trim_start_byte(item.name.value(), b'\\')),
                            namespace,
                        });
                    }
                }
                UseItems::MixedList(list) => {
                    let prefix = trim_start_byte(list.namespace.value(), b'\\');
                    for i in list.items.nodes {
                        let import_type = match i.r#type.as_ref() {
                            Some(t) if t.is_function() => ImportType::Function,
                            Some(t) if t.is_const() => ImportType::Constant,
                            _ => ImportType::ClassOrNamespace,
                        };
                        let fqn = concat_word!(prefix, b"\\", i.item.name.value());
                        declarations.push(UseDeclaration {
                            parent_stmt: stmt,
                            item: &i.item,
                            import_type,
                            fqn,
                            namespace,
                        });
                    }
                }
                UseItems::TypedList(list) => {
                    let prefix = trim_start_byte(list.namespace.value(), b'\\');
                    let import_type =
                        if list.r#type.is_function() { ImportType::Function } else { ImportType::Constant };
                    for item in list.items.nodes {
                        let fqn = concat_word!(prefix, b"\\", item.name.value());
                        declarations.push(UseDeclaration { parent_stmt: stmt, item, import_type, fqn, namespace });
                    }
                }
            }
        }
    }

    pub(super) fn is_item_used(
        decl: &UseDeclaration<'_>,
        used_fqns: &WordSet,
        docblocks: &Vec<&[u8]>,
        inline_contents: &Vec<&[u8]>,
    ) -> bool {
        let alias = get_alias(decl.item);

        if docblocks.iter().any(|doc| contains_word(doc, alias.as_bytes())) {
            return true;
        }

        if inline_contents.iter().any(|content| contains_word(content, alias.as_bytes())) {
            return true;
        }

        if used_fqns.iter().any(|used| used.as_bytes().eq_ignore_ascii_case(decl.fqn.as_bytes())) {
            return true;
        }

        if decl.import_type == ImportType::ClassOrNamespace {
            let prefix = concat_word!(decl.fqn, b"\\");
            if used_fqns.iter().any(|used| starts_with_ignore_case(used.as_bytes(), prefix.as_bytes())) {
                return true;
            }
        }

        false
    }

    /// Checks if `haystack` contains `needle` as a whole word (not as a substring of a larger identifier).
    ///
    /// A match is considered a whole word if the bytes immediately before and after the match
    /// are not ASCII alphanumeric or underscore (i.e., not PHP identifier bytes).
    fn contains_word(haystack: &[u8], needle: &[u8]) -> bool {
        let needle_len = needle.len();
        if needle_len == 0 || haystack.len() < needle_len {
            return false;
        }

        let mut start = 0;
        while let Some(pos) = memchr::memmem::find(&haystack[start..], needle) {
            let pos = start + pos;
            let before_ok = pos == 0 || !haystack[pos - 1].is_ascii_alphanumeric() && haystack[pos - 1] != b'_';
            let after_pos = pos + needle_len;
            let after_ok = after_pos >= haystack.len()
                || !haystack[after_pos].is_ascii_alphanumeric() && haystack[after_pos] != b'_';
            if before_ok && after_ok {
                return true;
            }
            start = pos + 1;
        }
        false
    }

    /// Check if an import is from the same namespace it appears in.
    ///
    /// Returns `true` if:
    /// - Both are in root namespace (import has no backslash, current namespace is None)
    /// - Import's parent namespace matches the current namespace
    pub(super) fn is_same_namespace_import(decl: &UseDeclaration<'_>) -> bool {
        let fqn = decl.fqn.as_bytes();

        let import_namespace = memchr::memrchr(b'\\', fqn).map(|pos| &fqn[..pos]);

        let is_same_namespace = match (&decl.namespace, import_namespace) {
            (None, None) => true,
            (Some(ns), Some(import_ns)) => ns.as_bytes().eq_ignore_ascii_case(import_ns),
            _ => false,
        };

        if !is_same_namespace {
            return false;
        }

        match &decl.item.alias {
            Some(alias) => {
                let name_bytes = decl.item.name.value();
                let last_segment = memchr::memrchr(b'\\', name_bytes).map_or(name_bytes, |i| &name_bytes[i + 1..]);
                alias.identifier.value.eq_ignore_ascii_case(last_segment)
            }
            None => true,
        }
    }

    pub(super) fn get_docblocks<'arena>(program: &Program<'arena>) -> Vec<&'arena [u8]> {
        program.trivia.iter().filter(|t| t.kind.is_docblock()).map(|t| t.value).collect()
    }

    pub(super) fn get_inline_contents<'arena>(program: &Program<'arena>) -> Vec<&'arena [u8]> {
        struct InlineWalker<'arena> {
            contents: Vec<&'arena [u8]>,
        }

        impl<'arena> MutWalker<'_, 'arena, ()> for InlineWalker<'arena> {
            fn walk_in_inline(&mut self, inline: &'_ Inline<'arena>, (): &mut ()) {
                self.contents.push(inline.value);
            }
        }

        let mut walker = InlineWalker { contents: Vec::new() };
        walker.walk_program(program, &mut ());
        walker.contents
    }

    pub(super) fn build_used_fqn_set<A>(ctx: &LintContext<'_, '_, A>, declarations: &[UseDeclaration<'_>]) -> WordSet
    where
        A: Arena,
    {
        let import_starts: HashSet<u32> = declarations.iter().map(|d| d.item.name.span().start.offset).collect();

        ctx.resolved_names
            .iter()
            .filter(|(start, _, _, _)| !import_starts.contains(start))
            .map(|(_, _, fqn, _)| word(fqn))
            .collect()
    }

    pub(super) fn get_alias(item: &UseItem) -> Word {
        let bytes = item.alias.as_ref().map_or_else(
            || {
                let name_bytes = item.name.value();
                memchr::memrchr(b'\\', name_bytes).map_or(name_bytes, |i| &name_bytes[i + 1..])
            },
            |alias| alias.identifier.value,
        );
        word(bytes)
    }

    pub(super) fn calculate_delete_range_for_item(
        parent_stmt: &Statement,
        item_to_delete: &UseItem,
        file_id: FileId,
    ) -> Option<Span> {
        let Statement::Use(use_stmt) = parent_stmt else { return None };

        let items = match &use_stmt.items {
            UseItems::Sequence(s) => &s.items,
            UseItems::TypedSequence(s) => &s.items,
            UseItems::TypedList(l) => &l.items,
            UseItems::MixedList(l) => return Some(find_range_in_mixed_list(l, item_to_delete, file_id)),
        };

        let Some(index) = items.nodes.iter().position(|i| std::ptr::eq(i, item_to_delete)) else {
            return Some(item_to_delete.span());
        };

        if items.nodes.len() == 1 {
            return Some(parent_stmt.span());
        }

        let delete_span = if index > 0 {
            let comma_span = items.tokens[index - 1].span_for(file_id);
            comma_span.join(item_to_delete.span())
        } else {
            let comma_span = items.tokens[index].span_for(file_id);
            item_to_delete.span().join(comma_span)
        };

        Some(delete_span)
    }

    fn find_range_in_mixed_list(list: &MixedUseItemList, item_to_delete: &UseItem, file_id: FileId) -> Span {
        let Some(index) = list.items.nodes.iter().position(|i| std::ptr::eq(&raw const i.item, item_to_delete)) else {
            return item_to_delete.span();
        };

        if list.items.nodes.len() == 1 {
            return list.span();
        }

        let typed_item_span = list.items.nodes[index].span();

        if index > 0 {
            let comma_span = list.items.tokens[index - 1].span_for(file_id);
            comma_span.join(typed_item_span)
        } else {
            let comma_span = list.items.tokens[index].span_for(file_id);
            typed_item_span.join(comma_span)
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoRedundantUseRule;
    use crate::integration::Integration;
    use crate::integration::IntegrationSet;
    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = used_import_is_not_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace App;

            use App\Helpers\ArrayHelper;

            $result = ArrayHelper::combine([]);
        "}
    }

    test_lint_success! {
        name = used_import_with_leading_backslash_is_not_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace Example;

            use \DOMDocument;

            $_ = new DOMDocument();
        "}
    }

    test_lint_success! {
        name = multiple_used_imports_with_leading_backslash,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace Example;

            use \DOMDocument;
            use \DOMElement;

            $doc = new DOMDocument();
            $elem = new DOMElement('div');
        "}
    }

    test_lint_success! {
        name = function_import_with_leading_backslash_is_not_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace Example;

            use function \array_map;

            $_ = array_map(fn($x) => $x, []);
        "}
    }

    test_lint_success! {
        name = const_import_with_leading_backslash_is_not_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace Example;

            use const \PHP_VERSION;

            $_ = PHP_VERSION;
        "}
    }

    test_lint_success! {
        name = tempest_inline_usage,
        rule = NoRedundantUseRule,
        filename = "test.view.php",
        settings = |settings: &mut Settings| {
            settings.integrations = IntegrationSet::only(Integration::Tempest);
        },
        code = indoc! {r#"
            <?php

            use Tests\Tempest\Fixtures\Modules\Home\HomeController;
            use function Tempest\Router\uri;

            ?>

            {{ uri(HomeController::class) }}
        "#}
    }

    test_lint_failure! {
        name = unused_import_is_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace App;

            use App\Helpers\StringHelper;

            $result = [];
        "}
    }

    test_lint_failure! {
        name = unused_import_with_leading_backslash_is_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace Example;

            use \DOMDocument;

            $_ = 'no usage';
        "}
    }

    test_lint_failure! {
        name = unused_function_import_with_leading_backslash_is_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace Example;

            use function \array_map;

            $_ = [];
        "}
    }

    test_lint_failure! {
        name = unused_const_import_with_leading_backslash_is_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace Example;

            use const \PHP_VERSION;

            $_ = 1;
        "}
    }

    test_lint_failure! {
        name = partially_unused_imports,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace App;

            use App\Helpers\ArrayHelper;
            use App\Helpers\StringHelper;

            $result = ArrayHelper::combine([]);
        "}
    }

    test_lint_success! {
        name = different_namespace_import_is_not_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace Foo\Bar;

            use Foo\Baz\Qux;

            $_ = new Qux();
        "}
    }

    test_lint_success! {
        name = root_import_in_namespace_is_not_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace App;

            use RuntimeException;

            throw new RuntimeException('error');
        "}
    }

    test_lint_success! {
        name = parent_namespace_import_is_not_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace Foo\Bar\Baz;

            use Foo\Bar\Qux;

            $_ = new Qux();
        "}
    }

    test_lint_success! {
        name = same_namespace_import_with_alias_is_not_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace Foo\Bar;

            use Foo\Bar\Baz as Qux;

            $_ = new Qux();
        "}
    }

    test_lint_failure! {
        name = same_namespace_import_is_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace Foo\Bar;

            use Foo\Bar\Baz;

            $_ = new Baz();
        "}
    }

    test_lint_failure! {
        name = same_namespace_function_import_is_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace Foo\Bar;

            use function Foo\Bar\qux;

            qux();
        "}
    }

    test_lint_failure! {
        name = same_namespace_const_import_is_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace Foo\Bar;

            use const Foo\Bar\QUXX;

            echo QUXX;
        "}
    }

    test_lint_failure! {
        name = root_namespace_class_import_is_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            use Foo;

            $_ = new Foo();
        "}
    }

    test_lint_failure! {
        name = root_namespace_function_import_is_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            use function strlen;

            echo strlen('hello');
        "}
    }

    test_lint_failure! {
        name = root_namespace_const_import_is_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            use const PHP_VERSION;

            echo PHP_VERSION;
        "}
    }

    test_lint_failure! {
        name = braced_root_namespace_import_is_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace {
                use RuntimeException;

                throw new RuntimeException('error');
            }
        "}
    }

    test_lint_failure! {
        name = braced_root_namespace_function_import_is_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace {
                use function strlen;

                echo strlen('a');
            }
        "}
    }

    test_lint_failure! {
        name = braced_root_namespace_const_import_is_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace {
                use const PHP_VERSION_ID;

                echo PHP_VERSION_ID;
            }
        "}
    }

    test_lint_failure! {
        name = same_namespace_import_case_insensitive,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace foo\bar;

            use Foo\Bar\Baz;

            $_ = new Baz();
        "}
    }

    test_lint_failure! {
        name = same_namespace_import_with_same_name_alias_is_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace Foo\Bar;

            use Foo\Bar\Baz as Baz;

            $_ = new Baz();
        "}
    }

    test_lint_failure! {
        name = root_namespace_import_with_same_name_alias_is_redundant,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            use Foo as Foo;

            $_ = new Foo();
        "}
    }

    test_lint_failure! {
        name = import_not_used_despite_substring_in_docblock,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace App\Commands;

            use Config;

            class TestCommand
            {
                public function handle(): void
                {
                    /**
                     * @var Collection<int, ConfigUsage> $usages
                     */
                    $usages = new Collection();
                }
            }
        "}
    }

    test_lint_success! {
        name = import_used_in_docblock_as_whole_word,
        rule = NoRedundantUseRule,
        code = indoc! {r"
            <?php

            namespace App\Commands;

            use Config;

            class TestCommand
            {
                public function handle(): void
                {
                    /**
                     * @var Config $config
                     */
                    $config = null;
                }
            }
        "}
    }
}
