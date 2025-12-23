use std::collections::HashMap;

use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::AtomSet;
use mago_atom::atom;
use mago_atom::starts_with_ignore_case;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Inline;
use mago_syntax::ast::MixedUseItemList;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Program;
use mago_syntax::ast::Statement;
use mago_syntax::ast::UseItem;
use mago_syntax::ast::UseItems;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoRedundantUseRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantUseConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
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
                Detects `use` statements that import items that are never used.
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

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Program(program) = node else { return };

        let mut check_inline_mentions = false;

        // If `tempest` integration is enabled, and this file ends with `.view.php`,
        // check inline mentions as well.
        if ctx.registry.is_integration_enabled(Integration::Tempest)
            && ctx.source_file.path.as_ref().and_then(|p| p.to_str()).is_some_and(|s| s.ends_with(".view.php"))
        {
            check_inline_mentions = true;
        }

        let use_declarations = utils::collect_use_declarations(program);
        if use_declarations.is_empty() {
            return;
        }

        let used_fqns = utils::build_used_fqn_set(ctx);
        let docblocks = utils::get_docblocks(program);
        let inline_contents =
            if check_inline_mentions { utils::get_inline_contents(program) } else { Vec::with_capacity(0) };

        let grouped_by_parent = use_declarations.into_iter().fold(HashMap::new(), |mut acc, decl| {
            acc.entry(decl.parent_stmt.span()).or_insert_with(Vec::new).push(decl);
            acc
        });

        for decls in grouped_by_parent.values() {
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
                    let issue = Issue::new(self.cfg.level(), format!("Unused import: `{alias}`."))
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(unused_decl.item.name.span())
                                .with_message(format!("`{alias}` is imported but never used.")),
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
                    issue = issue.with_annotation(
                        Annotation::primary(unused_decl.item.span())
                            .with_message(format!("`{alias}` is imported but never used.")),
                    );
                }

                ctx.collector.propose(issue, |edits| {
                    for unused_decl in unused_items.iter().rev() {
                        if let Some(delete_range) =
                            utils::calculate_delete_range_for_item(parent_stmt, unused_decl.item)
                        {
                            edits.push(TextEdit::delete(delete_range));
                        }
                    }
                });
            }
        }
    }
}

mod utils {
    use mago_atom::concat_atom;
    use mago_span::Span;
    use mago_syntax::walker::MutWalker;

    use super::Atom;
    use super::AtomSet;
    use super::HasSpan;
    use super::Inline;
    use super::LintContext;
    use super::MixedUseItemList;
    use super::Program;
    use super::Statement;
    use super::UseItem;
    use super::UseItems;
    use super::atom;
    use super::starts_with_ignore_case;

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
        pub fqn: Atom,
    }

    pub(super) fn collect_use_declarations<'ast>(program: &'ast Program<'ast>) -> Vec<UseDeclaration<'ast>> {
        let mut declarations = Vec::new();
        for stmt in &program.statements {
            if let Statement::Namespace(ns) = stmt {
                for ns_stmt in ns.statements() {
                    collect_from_statement(ns_stmt, &mut declarations);
                }
            } else {
                collect_from_statement(stmt, &mut declarations);
            }
        }
        declarations
    }

    fn collect_from_statement<'ast>(stmt: &'ast Statement<'ast>, declarations: &mut Vec<UseDeclaration<'ast>>) {
        if let Statement::Use(use_stmt) = stmt {
            match &use_stmt.items {
                UseItems::Sequence(s) => {
                    let import_type = ImportType::ClassOrNamespace;
                    for item in &s.items.nodes {
                        declarations.push(UseDeclaration {
                            parent_stmt: stmt,
                            item,
                            import_type,
                            fqn: atom(item.name.value().trim_start_matches('\\')),
                        });
                    }
                }
                UseItems::TypedSequence(s) => {
                    let import_type = if s.r#type.is_function() { ImportType::Function } else { ImportType::Constant };
                    for item in &s.items.nodes {
                        declarations.push(UseDeclaration {
                            parent_stmt: stmt,
                            item,
                            import_type,
                            fqn: atom(item.name.value().trim_start_matches('\\')),
                        });
                    }
                }
                UseItems::MixedList(list) => {
                    let prefix = list.namespace.value().trim_start_matches('\\');
                    for i in &list.items.nodes {
                        let import_type = match i.r#type.as_ref() {
                            Some(t) if t.is_function() => ImportType::Function,
                            Some(t) if t.is_const() => ImportType::Constant,
                            _ => ImportType::ClassOrNamespace,
                        };
                        let fqn = concat_atom!(prefix, "\\", i.item.name.value());
                        declarations.push(UseDeclaration { parent_stmt: stmt, item: &i.item, import_type, fqn });
                    }
                }
                UseItems::TypedList(list) => {
                    let prefix = list.namespace.value().trim_start_matches('\\');
                    let import_type =
                        if list.r#type.is_function() { ImportType::Function } else { ImportType::Constant };
                    for item in &list.items.nodes {
                        let fqn = concat_atom!(prefix, "\\", item.name.value());
                        declarations.push(UseDeclaration { parent_stmt: stmt, item, import_type, fqn });
                    }
                }
            }
        }
    }

    pub(super) fn is_item_used(
        decl: &UseDeclaration<'_>,
        used_fqns: &AtomSet,
        docblocks: &Vec<&str>,
        inline_contents: &Vec<&str>,
    ) -> bool {
        let alias = get_alias(decl.item);

        if docblocks.iter().any(|doc| doc.contains(alias.as_str())) {
            return true;
        }

        if inline_contents.iter().any(|content| content.contains(alias.as_str())) {
            return true;
        }

        if used_fqns.iter().any(|used| used.eq_ignore_ascii_case(decl.fqn.as_str())) {
            return true;
        }

        if decl.import_type == ImportType::ClassOrNamespace {
            let prefix = concat_atom!(decl.fqn, "\\");
            if used_fqns.iter().any(|used| starts_with_ignore_case(used.as_str(), prefix.as_str())) {
                return true;
            }
        }

        false
    }

    pub(super) fn get_docblocks<'arena>(program: &Program<'arena>) -> Vec<&'arena str> {
        program.trivia.iter().filter(|t| t.kind.is_docblock()).map(|t| t.value).collect()
    }

    pub(super) fn get_inline_contents<'arena>(program: &Program<'arena>) -> Vec<&'arena str> {
        struct InlineWalker<'arena> {
            contents: Vec<&'arena str>,
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

    pub(super) fn build_used_fqn_set(ctx: &LintContext<'_, '_>) -> AtomSet {
        ctx.resolved_names.all().iter().map(|(_, (fqn, _))| atom(fqn)).collect()
    }

    pub(super) fn get_alias(item: &UseItem) -> Atom {
        atom(item.alias.as_ref().map_or_else(|| item.name.last_segment(), |alias| alias.identifier.value))
    }

    pub(super) fn calculate_delete_range_for_item(parent_stmt: &Statement, item_to_delete: &UseItem) -> Option<Span> {
        let Statement::Use(use_stmt) = parent_stmt else { return None };

        let items = match &use_stmt.items {
            UseItems::Sequence(s) => &s.items,
            UseItems::TypedSequence(s) => &s.items,
            UseItems::TypedList(l) => &l.items,
            UseItems::MixedList(l) => return Some(find_range_in_mixed_list(l, item_to_delete)),
        };

        let Some(index) = items.nodes.iter().position(|i| std::ptr::eq(i, item_to_delete)) else {
            return Some(item_to_delete.span());
        };

        if items.nodes.len() == 1 {
            return Some(parent_stmt.span());
        }

        let delete_span = if index > 0 {
            let comma_span = items.tokens[index - 1].span;
            comma_span.join(item_to_delete.span())
        } else {
            let comma_span = items.tokens[index].span;
            item_to_delete.span().join(comma_span)
        };

        Some(delete_span)
    }

    fn find_range_in_mixed_list(list: &MixedUseItemList, item_to_delete: &UseItem) -> Span {
        let Some(index) = list.items.nodes.iter().position(|i| std::ptr::eq(&raw const i.item, item_to_delete)) else {
            return item_to_delete.span();
        };

        if list.items.nodes.len() == 1 {
            return list.span();
        }

        let typed_item_span = list.items.nodes[index].span();

        if index > 0 {
            let comma_span = list.items.tokens[index - 1].span;
            comma_span.join(typed_item_span)
        } else {
            let comma_span = list.items.tokens[index].span;
            typed_item_span.join(comma_span)
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoRedundantUseRule;
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
}
