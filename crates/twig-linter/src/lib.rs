//! A rule-based linter for Twig templates.

#![allow(clippy::needless_raw_strings)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::exhaustive_enums)]
#![allow(clippy::pub_use)]

use std::sync::Arc;

use mago_allocator::prelude::*;
use mago_collector::Collector;
use mago_database::file::File;
use mago_reporting::IssueCollection;
use mago_twig_syntax::cst::Node;
use mago_twig_syntax::cst::Template;

use crate::context::LintContext;
use crate::registry::RuleRegistry;
use crate::rule::AnyRule;
use crate::scope::Scope;
use crate::settings::Settings;

pub mod category;
pub mod context;
pub mod registry;
pub mod requirements;
pub mod rule;
pub mod rule_meta;
pub mod scope;
pub mod settings;
pub mod version;

mod pragmas;

#[derive(Debug, Clone)]
pub struct Linter<'arena, A>
where
    A: Arena,
{
    arena: &'arena A,
    registry: Arc<RuleRegistry>,
}

impl<'arena, A> Linter<'arena, A>
where
    A: Arena,
{
    /// Creates a new Linter instance.
    ///
    /// # Arguments
    ///
    /// * `arena` - The bump allocator to use for memory management.
    /// * `settings` - The settings to use for configuring the linter.
    /// * `only` - If `Some`, only the rules with the specified codes will be loaded.
    ///   If `None`, all rules enabled by the settings will be loaded.
    /// * `include_disabled` - If `true`, includes rules that are disabled in the settings.
    pub fn new(arena: &'arena A, settings: &Settings, only: Option<&[String]>, include_disabled: bool) -> Self {
        Self { arena, registry: Arc::new(RuleRegistry::build(settings, only, include_disabled)) }
    }

    /// Creates a new Linter instance from an existing `RuleRegistry`.
    ///
    /// # Arguments
    ///
    /// * `arena` - The bump allocator to use for memory management.
    /// * `registry` - The rule registry to use for linting.
    pub fn from_registry(arena: &'arena A, registry: Arc<RuleRegistry>) -> Self {
        Self { arena, registry }
    }

    #[must_use]
    pub fn rules(&self) -> &[AnyRule] {
        self.registry.rules()
    }

    /// Lints a parsed template.
    ///
    /// The template must be free of syntax errors; report those before linting.
    #[must_use]
    pub fn lint(&self, source_file: &File, template: &Template<'arena>) -> IssueCollection {
        let pragmas = pragmas::collect(self.arena, source_file, template);
        let mut collector = Collector::from_pragmas(self.arena, source_file, pragmas);

        if let Some(only_codes) = &self.registry.only {
            collector.set_active_codes(only_codes);
        }

        let mut excluded_rules = Vec::new_in(self.arena);
        if let Ok(file_name) = std::str::from_utf8(source_file.name.as_ref()) {
            for (rule_index, _) in self.registry.rules().iter().enumerate() {
                let matcher = self.registry.excludes_for(rule_index);
                if !matcher.is_empty() && matcher.is_match(file_name) {
                    excluded_rules.push(rule_index);
                }
            }
        }

        let mut context = LintContext::new(self.arena, &self.registry, source_file, collector);

        let has_legacy_raw = walk(Node::Template(template), &mut context, excluded_rules.as_slice());

        let mut issues = context.collector.finish();
        if has_legacy_raw {
            issues.take_edits().for_each(drop);
        }

        issues
    }
}

fn walk<'ctx, 'arena, A>(
    root: Node<'ctx, 'arena>,
    ctx: &mut LintContext<'ctx, 'arena, A>,
    excluded_rules: &[usize],
) -> bool
where
    A: Arena,
{
    enum Op<'ctx, 'arena> {
        Enter(Node<'ctx, 'arena>),
        Exit { in_scope: bool, conditional_depth: usize },
    }

    let mut has_legacy_raw = false;
    let mut stack = Vec::with_capacity_in(64, ctx.arena);
    stack.push(Op::Enter(root));

    while let Some(op) = stack.pop() {
        match op {
            Op::Enter(node) => {
                if let Node::Verbatim(verbatim) = node {
                    has_legacy_raw |= verbatim.keyword.value == b"raw";
                }

                ctx.push_ancestor(node);

                let in_scope = if let Some(scope) = Scope::for_node(ctx.arena, node) {
                    ctx.scope.push(scope);
                    true
                } else {
                    false
                };

                let rules_to_run = ctx.registry.for_kind(node.kind());
                for &rule_index in rules_to_run {
                    if excluded_rules.contains(&rule_index) {
                        continue;
                    }

                    ctx.registry.rule(rule_index).check(ctx, node);
                }

                // Push exit before children so teardown happens after all descendants.
                stack.push(Op::Exit { in_scope, conditional_depth: ctx.conditional_depth });

                match node {
                    Node::Conditional(_) => ctx.conditional_depth += 1,
                    Node::ArrowFunction(_) => ctx.conditional_depth = 0,
                    _ => {}
                }

                // Push children in reverse so they are processed left-to-right.
                let start = stack.len();
                node.visit_children(|child| stack.push(Op::Enter(child)));
                stack[start..].reverse();
            }
            Op::Exit { in_scope, conditional_depth } => {
                ctx.conditional_depth = conditional_depth;

                if in_scope {
                    ctx.scope.pop();
                }

                ctx.pop_ancestor();
            }
        }
    }

    has_legacy_raw
}
