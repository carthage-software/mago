#![allow(clippy::needless_raw_strings)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::exhaustive_enums)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::pub_use)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::match_wildcard_for_single_variants)]

use std::sync::Arc;

use mago_allocator::prelude::*;
use mago_collector::Collector;
use mago_database::file::File;
use mago_names::ResolvedNames;
use mago_php_version::PHPVersion;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::ForBody;
use mago_syntax::cst::ForeachBody;
use mago_syntax::cst::IfBody;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::Program;
use mago_syntax::cst::Statement;
use mago_syntax::cst::SwitchBody;
use mago_syntax::cst::WhileBody;

use crate::context::LintContext;
use crate::external::ExternalLintError;
use crate::external::ExternalLinter;
use crate::external::LinterTransport;
use crate::registry::RuleRegistry;
use crate::rule::AnyRule;
use crate::scope::Scope;
use crate::settings::Settings;

pub mod category;
pub mod context;
pub mod external;
pub mod import_tracker;
pub mod integration;
pub mod registry;
pub mod requirements;
pub mod rule;
pub mod rule_meta;
pub mod scope;
pub mod settings;

const COLLECTOR_CATEGORIES: &[&str] = &["lint", "linter"];

#[derive(Debug, Clone)]
pub struct Linter<'arena, A>
where
    A: Arena,
{
    arena: &'arena A,
    registry: Arc<RuleRegistry>,
    php_version: PHPVersion,
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
    pub fn from_registry(arena: &'arena A, registry: Arc<RuleRegistry>, php_version: PHPVersion) -> Self {
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
        match self.lint_internal::<mago_extension::WorkerPool>(source_file, program, resolved_names, None) {
            Ok(issues) => issues,
            Err(_) => unreachable!("linting without external rules cannot fail"),
        }
    }

    /// Lints a file with both built-in rules and registered external rules.
    ///
    /// External issues pass through the same collector as built-in issues, so
    /// `@mago-ignore` and `@mago-expect` pragmas behave identically for both.
    ///
    /// # Errors
    ///
    /// Returns an error if a worker fails or sends an invalid linter response.
    pub fn lint_with_external<'ctx, 'ast>(
        &self,
        source_file: &'ctx File,
        program: &'ast Program<'arena>,
        resolved_names: &'ast ResolvedNames<'arena>,
        external: &ExternalLinter,
    ) -> Result<IssueCollection, ExternalLintError> {
        self.lint_internal(source_file, program, resolved_names, Some(external))
    }

    fn lint_internal<'ctx, 'ast, T>(
        &self,
        source_file: &'ctx File,
        program: &'ast Program<'arena>,
        resolved_names: &'ast ResolvedNames<'arena>,
        external: Option<&ExternalLinter<T>>,
    ) -> Result<IssueCollection, ExternalLintError>
    where
        T: LinterTransport,
    {
        let mut collector = Collector::new(self.arena, source_file, program, COLLECTOR_CATEGORIES);

        // Set active codes if --only filter was used
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

        let mut context =
            LintContext::new(self.php_version, self.arena, &self.registry, source_file, resolved_names, collector);

        let recovered = walk(Node::Program(program), &mut context, excluded_rules.as_slice());

        if let Some(external) = external {
            let external_issues = external.lint(source_file, program, resolved_names, self.registry.only.as_deref())?;
            context.collector.extend(external_issues);
        }

        let issues = context.collector.finish();
        if recovered.is_empty() {
            return Ok(issues);
        }

        Ok(issues.into_iter().filter(|issue| !is_within(issue, &recovered)).collect())
    }
}

fn is_within(issue: &Issue, recovered: &[(Span, Span)]) -> bool {
    issue.annotations.iter().find(|annotation| annotation.is_primary()).is_some_and(|primary| {
        recovered.iter().any(|&(statement, part)| {
            encloses(part, primary.span) || (encloses(primary.span, part) && encloses(statement, primary.span))
        })
    })
}

fn encloses(outer: Span, inner: Span) -> bool {
    outer.start.offset <= inner.start.offset && inner.end.offset <= outer.end.offset
}

fn collect_recovered<A>(statement: &Statement<'_>, recovered: &mut Vec<'_, (Span, Span), A>)
where
    A: Arena,
{
    let mut push = |part: Option<Span>| {
        if let Some(part) = part {
            recovered.push((statement.span(), part));
        }
    };

    match statement {
        Statement::If(r#if) => match &r#if.body {
            IfBody::Statement(body) => {
                push(recovered_head(r#if.r#if.span, r#if.right_parenthesis, Some(body.statement)));
                for clause in &body.else_if_clauses {
                    push(recovered_head(clause.elseif.span, clause.right_parenthesis, Some(clause.statement)));
                }

                if let Some(clause) = &body.else_clause {
                    push(missing_body(clause.r#else.span, clause.statement));
                }
            }
            IfBody::ColonDelimited(body) => {
                push(recovered_head(r#if.r#if.span, r#if.right_parenthesis, None));
                for clause in &body.else_if_clauses {
                    push(recovered_head(clause.elseif.span, clause.right_parenthesis, None));
                }
            }
        },
        Statement::While(r#while) => {
            let body = match r#while.body {
                WhileBody::Statement(body) => Some(body),
                WhileBody::ColonDelimited(_) => None,
            };

            push(recovered_head(r#while.r#while.span, r#while.right_parenthesis, body));
        }
        Statement::DoWhile(do_while) if do_while.right_parenthesis.length() == 0 => {
            push(Some(do_while.r#while.span.join(do_while.terminator.span())));
        }
        Statement::Switch(switch) => match &switch.body {
            SwitchBody::BraceDelimited(body) if body.left_brace.length() == 0 => {
                push(Some(switch.switch.span.join(body.span())));
            }
            _ => push(recovered_head(switch.switch.span, switch.right_parenthesis, None)),
        },
        Statement::For(r#for) => {
            if let ForBody::Statement(body) = r#for.body {
                push(missing_body(r#for.r#for.span, body));
            }
        }
        Statement::Foreach(foreach) => {
            if let ForeachBody::Statement(body) = foreach.body {
                push(missing_body(foreach.foreach.span, body));
            }
        }
        _ => {}
    }
}

fn recovered_head(keyword: Span, right_parenthesis: Span, body: Option<&Statement<'_>>) -> Option<Span> {
    body.and_then(|body| missing_body(keyword, body))
        .or_else(|| (right_parenthesis.length() == 0).then(|| keyword.join(right_parenthesis)))
}

fn missing_body(keyword: Span, body: &Statement<'_>) -> Option<Span> {
    is_missing(body).then(|| keyword.join(body.span()))
}

fn is_missing(statement: &Statement<'_>) -> bool {
    matches!(statement, Statement::Noop(span) if span.length() == 0)
}

fn is_constant_expression_context(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Attribute
            | NodeKind::FunctionLikeParameter
            | NodeKind::PropertyConcreteItem
            | NodeKind::ClassLikeConstantItem
            | NodeKind::ConstantItem
    )
}

fn walk<'ctx, 'arena, A>(
    root: Node<'ctx, 'arena>,
    ctx: &mut LintContext<'ctx, 'arena, A>,
    excluded_rules: &[usize],
) -> Vec<'arena, (Span, Span), A>
where
    A: Arena,
{
    enum Op<'ctx, 'arena> {
        Enter(Node<'ctx, 'arena>),
        Exit { in_scope: bool, in_constant_expression: bool },
    }

    let mut recovered = Vec::new_in(ctx.arena);
    let mut stack = Vec::with_capacity_in(64, ctx.arena);
    stack.push(Op::Enter(root));

    while let Some(op) = stack.pop() {
        match op {
            Op::Enter(node) => {
                if let Node::Statement(statement) = node {
                    collect_recovered(statement, &mut recovered);
                }

                ctx.push_ancestor(node);

                let in_scope = if let Some(scope) = Scope::for_node(ctx, node) {
                    ctx.scope.push(scope);
                    true
                } else {
                    false
                };

                let in_constant_expression = is_constant_expression_context(node.kind());
                if in_constant_expression {
                    ctx.constant_expression_depth += 1;
                }

                let rules_to_run = ctx.registry.for_kind(node.kind());
                for &rule_index in rules_to_run {
                    if excluded_rules.contains(&rule_index) {
                        continue;
                    }

                    ctx.registry.rule(rule_index).check(ctx, node);
                }

                // Push exit before children so teardown happens after all descendants.
                stack.push(Op::Exit { in_scope, in_constant_expression });

                // Push children in reverse so they are processed left-to-right.
                let start = stack.len();
                node.visit_children(|child| stack.push(Op::Enter(child)));
                stack[start..].reverse();
            }
            Op::Exit { in_scope, in_constant_expression } => {
                if in_constant_expression {
                    ctx.constant_expression_depth -= 1;
                }

                if in_scope {
                    ctx.scope.pop();
                }

                ctx.pop_ancestor();
            }
        }
    }

    recovered
}
