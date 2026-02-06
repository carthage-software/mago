use std::collections::HashSet;
use std::sync::Arc;

use bumpalo::Bump;
use mago_collector::Collector;
use mago_database::file::File;
use mago_names::ResolvedNames;
use mago_php_version::PHPVersion;
use mago_reporting::IssueCollection;
use mago_syntax::ast::Node;
use mago_syntax::ast::Program;

use crate::context::LintContext;
use crate::registry::RuleRegistry;
use crate::rule::AnyRule;
use crate::scope::Scope;
use crate::settings::Settings;

pub mod category;
pub mod context;
pub mod integration;
pub mod registry;
pub mod requirements;
pub mod rule;
pub mod rule_meta;
pub mod scope;
pub mod settings;

const COLLECTOR_CATEGORIES: &[&str] = &["lint", "linter"];

#[derive(Debug, Clone)]
pub struct Linter<'arena> {
    arena: &'arena Bump,
    registry: Arc<RuleRegistry>,
    php_version: PHPVersion,
}

impl<'arena> Linter<'arena> {
    /// Creates a new Linter instance.
    ///
    /// # Arguments
    ///
    /// * `arena` - The bump allocator to use for memory management.
    /// * `settings` - The settings to use for configuring the linter.
    /// * `only` - If `Some`, only the rules with the specified codes will be loaded.
    ///   If `None`, all rules enabled by the settings will be loaded.
    /// * `include_disabled` - If `true`, includes rules that are disabled in the settings.
    pub fn new(arena: &'arena Bump, settings: &Settings, only: Option<&[String]>, include_disabled: bool) -> Self {
        Self {
            arena,
            php_version: settings.php_version,
            registry: Arc::new(RuleRegistry::build(settings, only, include_disabled)),
        }
    }

    /// Creates a new Linter instance from an existing `RuleRegistry`.
    ///
    /// # Arguments
    ///
    /// * `arena` - The bump allocator to use for memory management.
    /// * `registry` - The rule registry to use for linting.
    /// * `php_version` - The PHP version to use for linting.
    pub fn from_registry(arena: &'arena Bump, registry: Arc<RuleRegistry>, php_version: PHPVersion) -> Self {
        Self { arena, registry, php_version }
    }

    #[must_use]
    pub fn rules(&self) -> &[AnyRule] {
        self.registry.rules()
    }

    #[must_use]
    pub fn lint<'ctx, 'ast>(
        &self,
        source_file: &'ctx File,
        program: &'ast Program<'arena>,
        resolved_names: &'ast ResolvedNames<'arena>,
    ) -> IssueCollection {
        let mut collector = Collector::new(self.arena, source_file, program, COLLECTOR_CATEGORIES);

        // Set active codes if --only filter was used
        if let Some(only_codes) = &self.registry.only {
            collector.set_active_codes(only_codes);
        }

        // Compute which rules are excluded for this file
        let file_name = source_file.name.as_ref();
        let excluded_rules: HashSet<usize> = self
            .registry
            .rules()
            .iter()
            .enumerate()
            .filter(|(idx, _)| {
                let excludes = self.registry.excludes_for(*idx);
                !excludes.is_empty() && is_file_excluded(file_name, excludes)
            })
            .map(|(idx, _)| idx)
            .collect();

        let mut context =
            LintContext::new(self.php_version, self.arena, &self.registry, source_file, resolved_names, collector);

        walk(Node::Program(program), &mut context, &excluded_rules);

        context.collector.finish()
    }
}

fn is_file_excluded(file_name: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|pattern| {
        if pattern.ends_with('/') {
            file_name.starts_with(pattern.as_str())
        } else {
            let dir_prefix = format!("{pattern}/");
            file_name.starts_with(&dir_prefix) || file_name == pattern
        }
    })
}

fn walk<'arena>(node: Node<'_, 'arena>, ctx: &mut LintContext<'_, 'arena>, excluded_rules: &HashSet<usize>) {
    let mut in_scope = false;
    if let Some(scope) = Scope::for_node(ctx, node) {
        ctx.scope.push(scope);

        in_scope = true;
    }

    let rules_to_run = ctx.registry.for_kind(node.kind());

    for &rule_index in rules_to_run {
        if excluded_rules.contains(&rule_index) {
            continue;
        }

        let rule = ctx.registry.rule(rule_index);

        rule.check(ctx, node);
    }

    for child in node.children() {
        walk(child, ctx, excluded_rules);
    }

    if in_scope {
        ctx.scope.pop();
    }
}
