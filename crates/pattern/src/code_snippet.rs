//! [`CodeSnippet`] impl for Mago.
//!
//! A code snippet is the engine's abstraction for "this backtick-delimited pattern is a
//! bunch of possible shape-patterns; any of them can match". For Mago we parse the snippet
//! into one or more [`Pattern<MagoQueryContext>`] alternatives (one per snippet context
//! wrapper that parsed successfully) and hold them here.

use grit_pattern_matcher::binding::Binding;
use grit_pattern_matcher::pattern::CodeSnippet;
use grit_pattern_matcher::pattern::DynamicPattern;
use grit_pattern_matcher::pattern::Matcher;
use grit_pattern_matcher::pattern::Pattern;
use grit_pattern_matcher::pattern::PatternName;
use grit_pattern_matcher::pattern::ResolvedPattern;
use grit_pattern_matcher::pattern::State;
use grit_util::AnalysisLogs;
use grit_util::error::GritResult;

use crate::context::MagoExecContext;
use crate::query_context::MagoQueryContext;
use crate::resolved_pattern::MagoResolvedPattern;

#[derive(Debug, Clone)]
pub struct MagoCodeSnippet {
    pub patterns: Vec<Pattern<MagoQueryContext>>,
    pub source: String,
    pub dynamic_snippet: Option<DynamicPattern<MagoQueryContext>>,
}

impl MagoCodeSnippet {
    pub fn new(patterns: Vec<Pattern<MagoQueryContext>>, source: String) -> Self {
        Self { patterns, source, dynamic_snippet: None }
    }
}

impl CodeSnippet<MagoQueryContext> for MagoCodeSnippet {
    fn patterns(&self) -> impl Iterator<Item = &Pattern<MagoQueryContext>> {
        self.patterns.iter()
    }

    fn dynamic_snippet(&self) -> Option<&DynamicPattern<MagoQueryContext>> {
        self.dynamic_snippet.as_ref()
    }
}

impl PatternName for MagoCodeSnippet {
    fn name(&self) -> &'static str {
        "MAGO_CODE_SNIPPET"
    }
}

impl Matcher<MagoQueryContext> for MagoCodeSnippet {
    fn execute<'a>(
        &'a self,
        binding: &MagoResolvedPattern<'a>,
        state: &mut State<'a, MagoQueryContext>,
        context: &'a MagoExecContext<'a>,
        logs: &mut AnalysisLogs,
    ) -> GritResult<bool> {
        let Some(_node) = binding.get_last_binding().and_then(Binding::singleton) else {
            return Ok(false);
        };

        for pattern in &self.patterns {
            let mut trial = state.clone();
            if pattern.execute(binding, &mut trial, context, logs)? {
                *state = trial;
                return Ok(true);
            }
        }
        Ok(false)
    }
}
