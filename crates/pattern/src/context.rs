//! Execution context for running a pattern against a parsed file.
//!
//! Grit's `ExecContext` holds definitions the engine needs during matching: pattern
//! definitions, predicate definitions, function definitions, the file registry, and the
//! language. For Mago's initial integration we keep it simple: no user-defined pattern
//! or function definitions yet, just the language binding and the file owner.

use grit_pattern_matcher::context::ExecContext;
use grit_pattern_matcher::context::QueryContext;
use grit_pattern_matcher::file_owners::FileOwners;
use grit_pattern_matcher::pattern::CallBuiltIn;
use grit_pattern_matcher::pattern::CallbackPattern;
use grit_pattern_matcher::pattern::GritFunctionDefinition;
use grit_pattern_matcher::pattern::Matcher;
use grit_pattern_matcher::pattern::Pattern;
use grit_pattern_matcher::pattern::PatternDefinition;
use grit_pattern_matcher::pattern::PredicateDefinition;
use grit_pattern_matcher::pattern::State;
use grit_util::AnalysisLogs;
use grit_util::error::GritPatternError;
use grit_util::error::GritResult;

use crate::language::MagoLanguage;
use crate::query_context::MagoQueryContext;
use crate::resolved_pattern::MagoResolvedPattern;
use crate::tree::MagoTree;

pub struct MagoExecContext<'a> {
    pub language: &'a MagoLanguage,
    pub file_owners: &'a FileOwners<MagoTree<'a>>,
    pub pattern_definitions: Vec<PatternDefinition<MagoQueryContext>>,
    pub predicate_definitions: Vec<PredicateDefinition<MagoQueryContext>>,
    pub function_definitions: Vec<GritFunctionDefinition<MagoQueryContext>>,
    pub ignore_limit: bool,
    pub name: Option<String>,
}

impl<'a> MagoExecContext<'a> {
    pub fn new(language: &'a MagoLanguage, file_owners: &'a FileOwners<MagoTree<'a>>) -> Self {
        Self {
            language,
            file_owners,
            pattern_definitions: Vec::new(),
            predicate_definitions: Vec::new(),
            function_definitions: Vec::new(),
            ignore_limit: false,
            name: None,
        }
    }
}

impl<'a> ExecContext<'a, MagoQueryContext> for MagoExecContext<'a> {
    fn pattern_definitions(&self) -> &[PatternDefinition<MagoQueryContext>] {
        &self.pattern_definitions
    }

    fn predicate_definitions(&self) -> &[PredicateDefinition<MagoQueryContext>] {
        &self.predicate_definitions
    }

    fn function_definitions(&self) -> &[GritFunctionDefinition<MagoQueryContext>] {
        &self.function_definitions
    }

    fn ignore_limit_pattern(&self) -> bool {
        self.ignore_limit
    }

    fn call_built_in(
        &self,
        _call: &'a CallBuiltIn<MagoQueryContext>,
        _context: &'a Self,
        _state: &mut State<'a, MagoQueryContext>,
        _logs: &mut AnalysisLogs,
    ) -> GritResult<MagoResolvedPattern<'a>> {
        Err(GritPatternError::new("built-in function calls are not supported by the Mago query engine"))
    }

    fn call_callback<'b>(
        &self,
        _call: &'a CallbackPattern,
        _context: &'a Self,
        _binding: &'b MagoResolvedPattern<'a>,
        _state: &mut State<'a, MagoQueryContext>,
        _logs: &mut AnalysisLogs,
    ) -> GritResult<bool> {
        Err(GritPatternError::new("callback patterns are not supported by the Mago query engine"))
    }

    fn load_file(
        &self,
        _file: &<MagoQueryContext as QueryContext>::File<'a>,
        _state: &mut State<'a, MagoQueryContext>,
        _logs: &mut AnalysisLogs,
    ) -> GritResult<bool> {
        Ok(true)
    }

    fn files(&self) -> &FileOwners<MagoTree<'a>> {
        self.file_owners
    }

    fn language(&self) -> &MagoLanguage {
        self.language
    }

    fn exec_step(
        &'a self,
        step: &'a Pattern<MagoQueryContext>,
        binding: &MagoResolvedPattern<'a>,
        state: &mut State<'a, MagoQueryContext>,
        logs: &mut AnalysisLogs,
    ) -> GritResult<bool> {
        step.execute(binding, state, self, logs)
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}
