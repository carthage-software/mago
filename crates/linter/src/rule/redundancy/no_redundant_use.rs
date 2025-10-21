use std::collections::HashMap;
use std::ops::Range;

use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::AtomSet;
use mago_atom::atom;
use mago_fixer::SafetyClassification;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
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
    fn default_enabled() -> bool {
        // TODO(azjezz): enable this rule by default in the next major release.
        false
    }

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
            good_example: indoc! {r#"
                <?php
                namespace App;

                use App\Helpers\ArrayHelper;

                $result = ArrayHelper::combine([]);
            "#},
            bad_example: indoc! {r#"
                <?php
                namespace App;

                use App\Helpers\ArrayHelper;
                use App\Helpers\StringHelper; // StringHelper is not used.

                $result = ArrayHelper::combine([]);
            "#},
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

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
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

        for (_, decls) in grouped_by_parent.iter() {
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
                    let issue = Issue::new(self.cfg.level(), format!("Unused import: `{}`.", alias))
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(unused_decl.item.name.span())
                                .with_message(format!("`{}` is imported but never used.", alias)),
                        )
                        .with_annotation(
                            Annotation::secondary(use_stmt.r#use.span()).with_message("Unused `use` statement."),
                        )
                        .with_help("Remove the entire `use` statement.");

                    ctx.collector.propose(issue, |plan| {
                        plan.delete(parent_stmt.span().to_range(), SafetyClassification::Safe);
                    });
                } else {
                    let issue = Issue::new(self.cfg.level(), "Redundant `use` statement.")
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(parent_stmt.span())
                                .with_message("All symbols imported here are unused."),
                        )
                        .with_help("Remove the entire `use` statement.");

                    ctx.collector.propose(issue, |plan| {
                        plan.delete(parent_stmt.span().to_range(), SafetyClassification::Safe);
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
                            .with_message(format!("`{}` is imported but never used.", alias)),
                    );
                }

                ctx.collector.propose(issue, |plan| {
                    for unused_decl in unused_items.iter().rev() {
                        if let Some(delete_range) =
                            utils::calculate_delete_range_for_item(parent_stmt, unused_decl.item)
                        {
                            plan.delete(delete_range, SafetyClassification::Safe);
                        }
                    }
                });
            }
        }
    }
}

mod utils {
    use mago_atom::concat_atom;
    use mago_syntax::walker::MutWalker;

    use super::*;

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
        for stmt in program.statements.iter() {
            if let Statement::Namespace(ns) = stmt {
                for ns_stmt in ns.statements().iter() {
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
                    for item in s.items.nodes.iter() {
                        declarations.push(UseDeclaration {
                            parent_stmt: stmt,
                            item,
                            import_type,
                            fqn: atom(item.name.value()),
                        });
                    }
                }
                UseItems::TypedSequence(s) => {
                    let import_type = if s.r#type.is_function() { ImportType::Function } else { ImportType::Constant };
                    for item in s.items.nodes.iter() {
                        declarations.push(UseDeclaration {
                            parent_stmt: stmt,
                            item,
                            import_type,
                            fqn: atom(item.name.value()),
                        });
                    }
                }
                UseItems::MixedList(list) => {
                    let prefix = list.namespace.value();
                    for i in list.items.nodes.iter() {
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
                    let prefix = list.namespace.value();
                    let import_type =
                        if list.r#type.is_function() { ImportType::Function } else { ImportType::Constant };
                    for item in list.items.nodes.iter() {
                        let fqn = concat_atom!(prefix, "\\", item.name.value());
                        declarations.push(UseDeclaration { parent_stmt: stmt, item, import_type, fqn });
                    }
                }
            };
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
            if used_fqns
                .iter()
                .any(|used| used.as_str().to_ascii_lowercase().starts_with(&prefix.as_str().to_ascii_lowercase()))
            {
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
            fn walk_in_inline(&mut self, inline: &'_ Inline<'arena>, _: &mut ()) {
                self.contents.push(inline.value);
            }
        }

        let mut walker = InlineWalker { contents: Vec::new() };
        walker.walk_program(program, &mut ());
        walker.contents
    }

    pub(super) fn build_used_fqn_set<'arena>(ctx: &LintContext<'_, 'arena>) -> AtomSet {
        ctx.resolved_names.all().iter().map(|(_, (fqn, _))| atom(fqn)).collect()
    }

    pub(super) fn get_alias(item: &UseItem) -> Atom {
        atom(item.alias.as_ref().map_or_else(|| item.name.last_segment(), |alias| alias.identifier.value))
    }

    pub(super) fn calculate_delete_range_for_item(
        parent_stmt: &Statement,
        item_to_delete: &UseItem,
    ) -> Option<Range<u32>> {
        let Statement::Use(use_stmt) = parent_stmt else { return None };

        let items = match &use_stmt.items {
            UseItems::Sequence(s) => &s.items,
            UseItems::TypedSequence(s) => &s.items,
            UseItems::TypedList(l) => &l.items,
            UseItems::MixedList(l) => return find_range_in_mixed_list(l, item_to_delete),
        };

        let Some(index) = items.nodes.iter().position(|i| std::ptr::eq(i, item_to_delete)) else {
            return Some(item_to_delete.span().to_range());
        };

        if items.nodes.len() == 1 {
            return Some(parent_stmt.span().to_range());
        }

        let delete_span = if index > 0 {
            let comma_span = items.tokens[index - 1].span;
            comma_span.join(item_to_delete.span())
        } else {
            let comma_span = items.tokens[index].span;
            item_to_delete.span().join(comma_span)
        };

        Some(delete_span.to_range())
    }

    fn find_range_in_mixed_list(list: &MixedUseItemList, item_to_delete: &UseItem) -> Option<Range<u32>> {
        let Some(index) = list.items.nodes.iter().position(|i| std::ptr::eq(&i.item, item_to_delete)) else {
            return Some(item_to_delete.span().to_range());
        };

        if list.items.nodes.len() == 1 {
            return Some(list.span().to_range());
        }

        let typed_item_span = list.items.nodes[index].span();

        let delete_span = if index > 0 {
            let comma_span = list.items.tokens[index - 1].span;
            comma_span.join(typed_item_span)
        } else {
            let comma_span = list.items.tokens[index].span;
            typed_item_span.join(comma_span)
        };

        Some(delete_span.to_range())
    }
}
