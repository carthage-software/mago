#![allow(clippy::too_many_arguments)]

use bumpalo::Bump;

use mago_codex::context::ScopeContext;
use mago_codex::metadata::CodebaseMetadata;
use mago_collector::Collector;
use mago_database::file::File;
use mago_names::ResolvedNames;
use mago_span::HasSpan;
use mago_syntax::ast::Program;

use crate::analysis_result::AnalysisResult;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::plugin::PluginRegistry;
use crate::plugin::context::HookContext;
use crate::plugin::hook::HookAction;
use crate::settings::Settings;
use crate::statement::analyze_statements;

pub mod analysis_result;
pub mod code;
pub mod error;
pub mod plugin;
pub mod settings;

mod analyzable;
mod artifacts;
mod assertion;
mod common;
mod context;
mod expression;
mod formula;
mod invocation;
mod reconciler;
mod resolver;
mod statement;
mod utils;
mod visibility;

const COLLECTOR_CATEGORIES: &[&str] = &["analysis", "analyzer", "analyser"];

#[derive(Debug)]
pub struct Analyzer<'ctx, 'ast, 'arena> {
    pub arena: &'arena Bump,
    pub source_file: &'ctx File,
    pub resolved_names: &'ast ResolvedNames<'arena>,
    pub codebase: &'ctx CodebaseMetadata,
    pub settings: Settings,
    pub plugin_registry: &'ctx PluginRegistry,
}

impl<'ctx, 'ast, 'arena> Analyzer<'ctx, 'ast, 'arena> {
    pub fn new(
        arena: &'arena Bump,
        source_file: &'ctx File,
        resolved_names: &'ast ResolvedNames<'arena>,
        codebase: &'ctx CodebaseMetadata,
        plugin_registry: &'ctx PluginRegistry,
        settings: Settings,
    ) -> Self {
        Self { arena, source_file, resolved_names, codebase, plugin_registry, settings }
    }

    pub fn analyze(
        &self,
        program: &'ast Program<'arena>,
        analysis_result: &mut AnalysisResult,
    ) -> Result<(), AnalysisError> {
        #[cfg(not(target_arch = "wasm32"))]
        let start_time = std::time::Instant::now();

        if !program.has_script() {
            #[cfg(not(target_arch = "wasm32"))]
            {
                analysis_result.time_in_analysis = start_time.elapsed();
            }

            return Ok(());
        }

        let statements = program.statements.as_slice();

        let mut collector = Collector::new(self.arena, self.source_file, program, COLLECTOR_CATEGORIES);
        if self.settings.diff {
            collector.set_skip_unfulfilled_expect(true);
        }

        let mut context = Context::new(
            self.arena,
            self.codebase,
            self.source_file,
            self.resolved_names,
            &self.settings,
            statements[0].span(),
            program.trivia.as_slice(),
            collector,
            self.plugin_registry,
        );

        let mut block_context = BlockContext::new(ScopeContext::new(), context.settings.register_super_globals);
        let mut artifacts = AnalysisArtifacts::new();

        if self.plugin_registry.has_program_hooks() {
            let mut hook_context =
                HookContext::new(context.codebase, context.resolved_names, &mut block_context, &mut artifacts);

            if let HookAction::Skip =
                self.plugin_registry.before_program(self.source_file, program, &mut hook_context)?
            {
                for reported in hook_context.take_issues() {
                    context.collector.report_with_code(reported.code, reported.issue);
                }

                context.finish(artifacts, analysis_result);

                #[cfg(not(target_arch = "wasm32"))]
                {
                    analysis_result.time_in_analysis = start_time.elapsed();
                }

                return Ok(());
            }

            for reported in hook_context.take_issues() {
                context.collector.report_with_code(reported.code, reported.issue);
            }
        }

        analyze_statements(statements, &mut context, &mut block_context, &mut artifacts)?;

        // Call after_program hooks
        if self.plugin_registry.has_program_hooks() {
            let mut hook_context =
                HookContext::new(context.codebase, context.resolved_names, &mut block_context, &mut artifacts);
            self.plugin_registry.after_program(self.source_file, program, &mut hook_context)?;
            for reported in hook_context.take_issues() {
                context.collector.report_with_code(reported.code, reported.issue);
            }
        }

        context.finish(artifacts, analysis_result);

        // Filter issues through registered issue filter hooks
        if self.plugin_registry.has_issue_filter_hooks() {
            analysis_result.issues = self.plugin_registry.filter_issues(
                self.source_file,
                std::mem::take(&mut analysis_result.issues),
                self.codebase,
            );
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            analysis_result.time_in_analysis = start_time.elapsed();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::collections::BTreeMap;

    use foldhash::HashSet;

    use mago_atom::AtomSet;
    use mago_codex::metadata::CodebaseMetadata;
    use mago_codex::populator::populate_codebase;
    use mago_codex::reference::SymbolReferences;
    use mago_codex::scanner::scan_program;
    use mago_database::file::File;
    use mago_names::resolver::NameResolver;
    use mago_syntax::parser::parse_file;

    use crate::Analyzer;
    use crate::analysis_result::AnalysisResult;
    use crate::code::IssueCode;
    use crate::plugin::PluginRegistry;
    use crate::settings::Settings;

    #[derive(Debug, Clone)]
    pub struct TestCase {
        name: &'static str,
        content: &'static str,
        settings: Settings,
        expected_issues: Vec<IssueCode>,
    }

    impl TestCase {
        pub fn new(name: &'static str, content: &'static str) -> Self {
            Self {
                name,
                content,
                settings: Settings {
                    find_unused_expressions: true,
                    find_unused_definitions: true,
                    ..Default::default()
                },
                expected_issues: vec![],
            }
        }

        pub fn settings(mut self, settings: Settings) -> Self {
            self.settings = settings;
            self
        }

        pub fn expect_success(mut self) -> Self {
            self.expected_issues = vec![];
            self
        }

        pub fn expect_issues(mut self, codes: Vec<IssueCode>) -> Self {
            self.expected_issues = codes;
            self
        }

        pub fn run(self) {
            run_test_case_inner(self);
        }
    }

    fn run_test_case_inner(config: TestCase) {
        let arena = bumpalo::Bump::new();
        let source_file = File::ephemeral(Cow::Borrowed(config.name), Cow::Borrowed(config.content));

        let program = parse_file(&arena, &source_file);
        assert!(!program.has_errors(), "Parse failed: {:?}", program.errors);

        let resolver = NameResolver::new(&arena);
        let resolved_names = resolver.resolve(program);
        let mut codebase = scan_program(&arena, &source_file, program, &resolved_names);
        let mut symbol_references = SymbolReferences::new();

        populate_codebase(&mut codebase, &mut symbol_references, AtomSet::default(), HashSet::default());

        let plugin_registry = PluginRegistry::with_library_providers();

        let mut analysis_result = AnalysisResult::new(symbol_references);
        let analyzer =
            Analyzer::new(&arena, &source_file, &resolved_names, &codebase, &plugin_registry, config.settings);

        let analysis_run_result = analyzer.analyze(program, &mut analysis_result);

        if let Err(err) = analysis_run_result {
            panic!("Test '{}': Expected analysis to succeed, but it failed with an error: {}", config.name, err);
        }

        verify_reported_issues(config.name, analysis_result, codebase, &config.expected_issues);
    }

    fn verify_reported_issues(
        test_name: &str,
        mut analysis_result: AnalysisResult,
        mut codebase: CodebaseMetadata,
        expected_issue_codes: &[IssueCode],
    ) {
        let mut actual_issues_collected = std::mem::take(&mut analysis_result.issues);

        actual_issues_collected.extend(codebase.take_issues(true));

        let actual_issues_count = actual_issues_collected.len();
        let mut expected_issue_counts: BTreeMap<&str, usize> = BTreeMap::new();
        for kind in expected_issue_codes {
            *expected_issue_counts.entry(kind.as_str()).or_insert(0) += 1;
        }

        let mut actual_issue_counts: BTreeMap<String, usize> = BTreeMap::new();
        for actual_issue in &actual_issues_collected {
            let Some(issue_code) = actual_issue.code.clone() else {
                panic!("Analyzer returned an issue with no code: {actual_issue:?}");
            };

            *actual_issue_counts.entry(issue_code).or_insert(0) += 1;
        }

        let mut discrepancies = Vec::new();

        for (actual_kind, &actual_count) in &actual_issue_counts {
            let expected_count = expected_issue_counts.get(actual_kind.as_str()).copied().unwrap_or(0);
            if actual_count > expected_count {
                discrepancies.push(format!(
                    "- Unexpected issue(s) of kind `{}`: found {}, expected {}.",
                    actual_kind.as_str(),
                    actual_count,
                    expected_count
                ));
            }
        }

        for (expected_kind, expected_count) in expected_issue_counts {
            let actual_count = actual_issue_counts.get(expected_kind).copied().unwrap_or(0);
            if actual_count < expected_count {
                discrepancies.push(format!(
                    "- Missing expected issue(s) of kind `{expected_kind}`: expected {expected_count}, found {actual_count}.",
                ));
            }
        }

        if !discrepancies.is_empty() {
            let mut panic_message = format!("Test '{test_name}' failed with issue discrepancies:\n");
            for d in discrepancies {
                panic_message.push_str(&format!("  {d}\n"));
            }

            panic!("{}", panic_message);
        }

        if expected_issue_codes.is_empty() && actual_issues_count != 0 {
            let mut panic_message = format!("Test '{test_name}': Expected no issues, but found:\n");
            for issue in actual_issues_collected {
                panic_message.push_str(&format!(
                    "  - Code: `{}`, Message: \"{}\"\n",
                    issue.code.unwrap_or_default(),
                    issue.message
                ));
            }

            panic!("{}", panic_message);
        }
    }

    #[macro_export]
    macro_rules! test_analysis {
        (name = $test_name:ident, code = $code_str:expr $(,)?) => {
            #[test]
            pub fn $test_name() {
                $crate::tests::TestCase::new(stringify!($test_name), $code_str).expect_success().run();
            }
        };
        (name = $test_name:ident, settings = $settings:expr, code = $code_str:expr $(,)?) => {
            #[test]
            pub fn $test_name() {
                $crate::tests::TestCase::new(stringify!($test_name), $code_str).settings($settings).expect_success().run();
            }
        };
        (name = $test_name:ident, code = $code_str:expr, issues = [$($issue_kind:expr),* $(,)?] $(,)?) => {
            #[test]
            pub fn $test_name() {
                $crate::tests::TestCase::new(stringify!($test_name), $code_str)
                    .expect_issues(vec![$($issue_kind),*])
                    .run();
            }
        };
        (name = $test_name:ident, settings = $settings:expr, code = $code_str:expr, issues = [$($issue_kind:expr),* $(,)?] $(,)?) => {
            #[test]
            pub fn $test_name() {
                $crate::tests::TestCase::new(stringify!($test_name), $code_str)
                    .settings($settings)
                    .expect_issues(vec![$($issue_kind),*])
                    .run();
            }
        };
    }
}
